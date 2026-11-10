use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    English,
    Thai,
}

pub fn build(
    mut translate: impl FnMut(u32, bool, Layout) -> Option<char>,
) -> Vec<(String, String)> {
    let mut pairs = BTreeSet::new();
    for scan in 1..=255 {
        for shifted in [false, true] {
            if let (Some(en), Some(th)) = (
                translate(scan, shifted, Layout::English),
                translate(scan, shifted, Layout::Thai),
            ) {
                if !en.is_control()
                    && !th.is_control()
                    && !en.is_whitespace()
                    && !th.is_whitespace()
                {
                    pairs.insert((en.to_string(), th.to_string()));
                }
            }
        }
    }
    pairs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scans_both_layers_filters_and_deduplicates() {
        let mut calls = 0;
        let pairs = build(|scan, shifted, layout| {
            calls += 1;
            match (scan, shifted, layout) {
                (1 | 2, false, Layout::English) => Some('a'),
                (1 | 2, false, Layout::Thai) => Some('ฟ'),
                (1, true, Layout::English) => Some('A'),
                (1, true, Layout::Thai) => Some('ฤ'),
                (3, _, _) => Some('\n'),
                (4, _, _) => Some(' '),
                (5, _, Layout::English) => Some('b'),
                _ => None,
            }
        });
        assert_eq!(calls, 255 * 2 * 2);
        assert_eq!(
            pairs,
            vec![("A".into(), "ฤ".into()), ("a".into(), "ฟ".into())]
        );
    }
}
