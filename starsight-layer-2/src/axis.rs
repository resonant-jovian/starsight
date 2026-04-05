use crate::scale::LinearScale;

pub struct Axis {
    pub scale: LinearScale,
    pub label: Option<String>,
    pub tick_positions: Vec<f64>,
    pub tick_labels: Vec<String>,
}

impl Axis {
    pub fn auto_from_data(values: &[f64], target_ticks: usize) -> Option<Self> {
        let dmin = values.iter().copied().fold(f64::INFINITY, f64::min);
        let dmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ticks = crate::tick::wilkinson_extended(dmin, dmax, target_ticks, true);
        let labels: Vec<String> = ticks.iter().map(|t| format!("{t}")).collect();
        Some(Self {
            scale: LinearScale { domain_min: ticks[0], domain_max: *ticks.last()? },
            label: None, tick_positions: ticks, tick_labels: labels,
        })
    }
}