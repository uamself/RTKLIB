#[derive(Debug, Clone, Copy)]
pub struct GnssTime {
    pub seconds: f64,
}

impl GnssTime {
    pub fn new(seconds: f64) -> Self {
        Self { seconds }
    }
}
