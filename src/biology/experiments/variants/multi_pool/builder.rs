use super::types::{MultiPoolExperimentLogger, MultiPoolExperimentSettings};

#[derive(Builder)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct MultiPoolExperiment {
    pub settings: MultiPoolExperimentSettings,
    _logger: Option<MultiPoolExperimentLogger>,
}
