use std::sync::atomic::{AtomicU64, Ordering};

/// A focus generation and permission bit in ONE atomic word. A stale UIA query
/// must never re-enable a newer password focus (including during publication).
#[derive(Default)]
pub struct Gate(AtomicU64);
impl Gate {
    pub fn invalidate(&self) -> u64 {
        let mut old = self.0.load(Ordering::SeqCst);
        loop {
            let next = (old | 1).wrapping_add(1);
            match self
                .0
                .compare_exchange_weak(old, next, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return next,
                Err(current) => old = current,
            }
        }
    }
    pub fn publish_safe(&self, generation: u64) -> bool {
        self.0
            .compare_exchange(
                generation,
                generation | 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
    }
    pub fn token(&self) -> Option<u64> {
        let v = self.0.load(Ordering::SeqCst);
        (v & 1 != 0).then_some(v)
    }
    pub fn valid(&self, token: u64) -> bool {
        self.token() == Some(token)
    }
    pub fn current(&self, generation: u64) -> bool {
        self.0.load(Ordering::SeqCst) == generation
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deny_until_queried_and_reject_stale_queries_and_actions() {
        let gate = Gate::default();
        assert!(gate.token().is_none());
        let first = gate.invalidate();
        assert!(gate.publish_safe(first));
        let action = gate.token().unwrap();
        assert!(gate.valid(action));
        let second = gate.invalidate();
        assert!(!gate.valid(action));
        assert!(!gate.publish_safe(first));
        assert!(gate.token().is_none());
        assert!(gate.current(second));
        assert!(gate.publish_safe(second));
        assert!(!gate.valid(action));
    }
    #[test]
    fn a_focus_change_cannot_be_overwritten_by_old_publication() {
        let gate = std::sync::Arc::new(Gate::default());
        let old = gate.invalidate();
        let child = gate.clone();
        std::thread::spawn(move || {
            child.invalidate();
        })
        .join()
        .unwrap();
        assert!(!gate.publish_safe(old));
        assert!(gate.token().is_none());
    }
}
