use crate::biology::experiments::types::ExperimentSimSettings;

use super::types::{
    MultiPoolExperimentLogger, MultiPoolExperimentSettings, MultiPoolLoggingSettings,
};

#[derive(Builder)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct MultiPoolExperiment {
    pub settings: super::types::MultiPoolExperimentSettings,
    _logger: Option<MultiPoolExperimentLogger>,
}
#[derive(Builder)]
#[builder(
    name = "MultiPoolExperimentSettingsBuilder",
    pattern = "owned",
    setter(strip_option),
    build_fn(skip)
)]
pub struct MultiPoolExperimentSettingsBuilderTemplate {
    pub max_iterations: u64,
    pub chemistry_key: String,
    pub experiment_key: String,
    pub logging_settings: Option<MultiPoolLoggingSettings>,
    pub evaluation_points_per_tick: usize,

    pub reference_sim_settings: ExperimentSimSettings,
    pub reference_fitness_calculation_key: String,
}

impl MultiPoolExperimentSettingsBuilder {
    pub fn build(self) -> MultiPoolExperimentSettings {
        MultiPoolExperimentSettings {
            max_iterations: self.max_iterations.unwrap(),
            chemistry_key: self.chemistry_key.unwrap().clone(),
            experiment_key: self.experiment_key.unwrap().clone(),
            logging_settings: self.logging_settings.unwrap_or(None).clone(),
            evaluation_points_per_tick: self.evaluation_points_per_tick.unwrap(),
            reference_sim_settings: self.reference_sim_settings.unwrap().clone(),
            reference_fitness_calculation_key: self
                .reference_fitness_calculation_key
                .unwrap()
                .clone(),
        }
    }
}
