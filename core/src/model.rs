//! KFM1 v1 (all numbers little-endian, no padding):
//! magic [u8;4] = KFM1; version u32 = 1; order u32 (1..=8);
//! alphabet length u32 (1..=256); for each symbol: UTF-8 byte length u32,
//! then that single Unicode scalar's bytes. Alphabet includes BEGIN U+0002
//! and END U+0003. Next: base uniform log-probability f64 (using the original
//! vocabulary size, which need not include BEGIN for an order-1 model).
//! For each order 1..=order: seen table, then context back-off table.
//! Each table: entry count u64, then sorted unique (key u64, log-value f32).
//! Keys concatenate alphabet indices, eight bits per symbol, first symbol
//! most significant. Length is implicit in the table's order (contexts n-1).
//! Seen values are fully interpolated log probabilities, NOT raw counts.
//! Context values are log(T/(C+T)); absent contexts back off with weight 1.
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug)]
struct Table(Vec<(u64, f32)>);
impl Table {
    fn get(&self, key: u64) -> Option<f64> {
        self.0
            .binary_search_by_key(&key, |e| e.0)
            .ok()
            .map(|i| self.0[i].1 as f64)
    }
}

#[derive(Debug)]
pub struct Model {
    order: usize,
    alphabet: HashMap<char, u8>,
    base_log: f64,
    seen: Vec<Table>,
    backoff: Vec<Table>,
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if n > self.0.len() {
            return Err("truncated KFM1 model".into());
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn table(&mut self, symbols: usize, alphabet_len: usize) -> Result<Table, String> {
        let count = u64::from_le_bytes(self.take(8)?.try_into().unwrap());
        if count > (self.0.len() / 12) as u64 {
            return Err("invalid KFM1 table length".into());
        }
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let key = u64::from_le_bytes(self.take(8)?.try_into().unwrap());
            let value = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
            if !value.is_finite()
                || value > 0.0
                || entries.last().is_some_and(|e: &(u64, f32)| e.0 >= key)
            {
                return Err("invalid KFM1 probability or table ordering".into());
            }
            let mut remaining = key;
            for _ in 0..symbols {
                if (remaining & 255) as usize >= alphabet_len {
                    return Err("invalid KFM1 symbol index".into());
                }
                remaining >>= 8;
            }
            if remaining != 0 {
                return Err("invalid KFM1 key length".into());
            }
            entries.push((key, value));
        }
        Ok(Table(entries))
    }
}

impl Model {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let bytes =
            fs::read(path.as_ref()).map_err(|e| format!("{}: {e}", path.as_ref().display()))?;
        Self::from_bytes(&bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut reader = Reader(bytes);
        if reader.take(4)? != b"KFM1" || reader.u32()? != 1 {
            return Err("unsupported KFM1 format".into());
        }
        let order = reader.u32()? as usize;
        let size = reader.u32()? as usize;
        if !(1..=8).contains(&order) || !(1..=256).contains(&size) {
            return Err("invalid KFM1 order/alphabet".into());
        }
        let mut alphabet = HashMap::new();
        for index in 0..size {
            let len = reader.u32()? as usize;
            let symbol = std::str::from_utf8(reader.take(len)?).map_err(|e| e.to_string())?;
            let mut chars = symbol.chars();
            let c = chars.next().ok_or("empty KFM1 symbol")?;
            if chars.next().is_some() || alphabet.insert(c, index as u8).is_some() {
                return Err("invalid KFM1 alphabet".into());
            }
        }
        if !alphabet.contains_key(&'\u{2}') || !alphabet.contains_key(&'\u{3}') {
            return Err("missing KFM1 boundary markers".into());
        }
        let base_log = f64::from_le_bytes(reader.take(8)?.try_into().unwrap());
        if !base_log.is_finite() || base_log > 0.0 {
            return Err("invalid KFM1 base probability".into());
        }
        let (mut seen, mut backoff) = (Vec::new(), Vec::new());
        for n in 1..=order {
            seen.push(reader.table(n, size)?);
            backoff.push(reader.table(n - 1, size)?);
        }
        if !reader.0.is_empty() {
            return Err("trailing KFM1 bytes".into());
        }
        Ok(Self {
            order,
            alphabet,
            base_log,
            seen,
            backoff,
        })
    }

    pub fn logprob(&self, s: &str, complete: bool) -> f64 {
        // None is an unseen symbol. Never alias it to a real alphabet entry.
        let mut seq = vec![self.alphabet.get(&'\u{2}').copied(); self.order - 1];
        seq.extend(s.chars().map(|c| self.alphabet.get(&c).copied()));
        if complete {
            seq.push(self.alphabet.get(&'\u{3}').copied());
        }
        let mut answer = 0.0;
        for i in self.order - 1..seq.len() {
            let mut logp = self.base_log;
            for n in 1..=self.order {
                let context = &seq[i + 1 - n..i];
                let key = context
                    .iter()
                    .try_fold(0u64, |key, c| c.map(|c| (key << 8) | c as u64));
                if let Some(key) = key {
                    let gram = seq[i].map(|c| (key << 8) | c as u64);
                    if let Some(value) = gram.and_then(|g| self.seen[n - 1].get(g)) {
                        logp = value;
                    } else if let Some(weight) = self.backoff[n - 1].get(key) {
                        logp += weight;
                    }
                }
            }
            // Matches Python's max(p, 1e-300), including for very long inputs.
            answer += logp.max(-690.7755278982137);
        }
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_models() {
        for bytes in [b"".as_slice(), b"KFM1", b"KFM1\x02\0\0\0"] {
            assert!(Model::from_bytes(bytes).is_err());
        }
        let fixture = include_bytes!("../tests/fixtures/tiny.kfm");
        for end in 0..fixture.len() {
            assert!(Model::from_bytes(&fixture[..end]).is_err());
        }
        let mut trailing = fixture.to_vec();
        trailing.push(0);
        assert!(Model::from_bytes(&trailing).is_err());
    }
}
