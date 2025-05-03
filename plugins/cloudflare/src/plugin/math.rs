use super::config::Weight;

pub struct WeightCalc;

impl WeightCalc {
    pub fn calc_from(x: u32, values: &Weight) -> u16 {
        Self::calc(x, values.max, values.k, values.a)
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn calc(x: u32, max: f64, k: f64, a: f64) -> u16 {
        let numerator = (-k * (f64::from(x) / max)).exp() - (-k).exp();
        let denominator = 1.0 - (-k).exp();
        let weight = a * max * (numerator / denominator);

        weight.round().clamp(0.0, f64::from(u16::MAX)) as u16
    }
}
