use super::{construct_chemistry, ChemistryConfiguration, ChemistryInstance};

/**
 * Stores everything needed to create a new chemistry instance. (similar to a builder)
 */
pub struct ChemistryBuilder {
    pub chemistry_key: String,
    pub chemistry_configuration: Option<ChemistryConfiguration>,
}

impl ChemistryBuilder {
    pub fn with_key(chemistry_key: &str) -> Self {
        Self {
            chemistry_key: chemistry_key.to_owned(),
            chemistry_configuration: None,
        }
    }

    pub fn config(mut self, config: ChemistryConfiguration) -> Self {
        self.chemistry_configuration = Some(config);
        self
    }

    pub fn build(&self) -> ChemistryInstance {
        construct_chemistry(&self.chemistry_key, self.chemistry_configuration.clone())
    }
}
