use crate::*;

pub trait F32Extensions {
    fn signum_zero(self) -> Self;
    fn spread(self, amount: f32) -> Self;
    fn spread_in(self, amount: f32) -> Self;
    fn spread_zero(self, amount: f32) -> Self;
    fn clamp_scale(self, from: Range<f32>, to: Range<f32>) -> f32;
}

impl F32Extensions for f32 {
    fn signum_zero(self) -> Self {
        if self == 0.0 {
            0.0
        } else {
            self.signum()
        }
    }

    /// 0.2 => 0.9..1.1
    fn spread(self, amount: f32) -> Self {
        1.0 + self.spread_zero(amount)
    }

    /// 0.2 => 0.8..1.0
    fn spread_in(self, amount: f32) -> Self {
        1.0 + self * amount - amount
    }

    fn spread_zero(self, amount: f32) -> Self {
        self * amount - amount / 2.0
    }

    fn clamp_scale(self, from: Range<f32>, to: Range<f32>) -> f32 {
        let clamped = self.clamp(from.start, from.end);
        let pct = (clamped - from.start) / (from.end - from.start);
        to.start + pct * (to.end - to.start)
    }
}

#[test]
pub fn spread_test() {
    assert_eq!(1.0.spread(0.0), 1.0);
}

#[test]
pub fn clamp_scale_test() {
    assert_eq!(0.5.clamp_scale(0.0..1.0, 0.0..2.0), 1.0);
    assert_eq!(0.0.clamp_scale(-1.0..1.0, 0.0..2.0), 1.0);
}
