pub trait Scale {
    fn map(&self, value: f64) -> f64;
    fn inverse(&self, normalized: f64) -> f64;
}

pub struct LinearScale {
    pub domain_min: f64,
    pub domain_max: f64,
}

impl Scale for LinearScale {
    fn map(&self, value: f64) -> f64 {
        if (self.domain_max - self.domain_min).abs() < f64::EPSILON {
            return 0.5;
        }
        (value - self.domain_min) / (self.domain_max - self.domain_min)
    }
    fn inverse(&self, normalized: f64) -> f64 {
        normalized * (self.domain_max - self.domain_min) + self.domain_min
    }
}
