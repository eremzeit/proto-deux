use super::{gene_pool::ExperimentGenePool, MultiPoolExperiment};

pub trait MultiPoolExperimentDataStore {
    fn save_snapshot(&mut self, experiment: &MultiPoolExperiment);
    fn load_snapshot(&mut self, experiment_key: &str) -> MultiPoolExperiment;

    fn update_genepool(&mut self, genepool: ExperimentGenePool);
}
