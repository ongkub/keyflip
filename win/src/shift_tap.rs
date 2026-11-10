/// All times are monotonic milliseconds supplied by the caller. Auto-repeat,
/// overlapping Shift keys, shortcuts and clicks invalidate the entire sequence.
#[derive(Default)]
pub struct ShiftTap {
    down: Option<(u32, u64)>,
    last_up: Option<u64>,
}
impl ShiftTap {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn event(&mut self, vk: u32, down: bool, now: u64) -> bool {
        if down {
            if self.down.is_some() {
                self.clear();
            } else {
                self.down = Some((vk, now));
            }
            return false;
        }
        let Some((key, start)) = self.down.take() else {
            return false;
        };
        if key != vk || now.saturating_sub(start) >= 250 {
            self.clear();
            return false;
        }
        if self
            .last_up
            .take()
            .is_some_and(|last| now.saturating_sub(last) <= 400)
        {
            true
        } else {
            self.last_up = Some(now);
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clean_taps_and_boundaries() {
        let mut s = ShiftTap::default();
        assert!(!s.event(160, true, 0));
        assert!(!s.event(160, false, 100));
        assert!(!s.event(161, true, 400));
        assert!(s.event(161, false, 500));
        assert!(!s.event(160, true, 600));
        assert!(!s.event(160, false, 601));
        s.event(160, true, 700);
        assert!(!s.event(160, false, 950));
        s.event(160, true, 1000);
        assert!(!s.event(160, false, 1001));
    }
    #[test]
    fn interruptions_repeat_overlap_and_timeout() {
        for interrupted in 0..4 {
            let mut s = ShiftTap::default();
            s.event(160, true, 0);
            s.event(160, false, 1);
            match interrupted {
                0 => s.clear(), // Any other key-down or click.
                1 => {
                    s.event(160, true, 10);
                    s.event(160, true, 11);
                    s.event(160, false, 12);
                }
                2 => {
                    s.event(160, true, 10);
                    s.event(161, true, 11);
                    s.event(160, false, 12);
                }
                _ => {
                    s.event(160, true, 450);
                }
            }
            if interrupted != 3 {
                s.event(160, true, 20);
            }
            assert!(!s.event(160, false, if interrupted == 3 { 451 } else { 21 }));
        }
    }
}
