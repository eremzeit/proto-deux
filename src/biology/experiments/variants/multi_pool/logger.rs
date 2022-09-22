use crate::biology::experiments::alterations;
use crate::biology::experiments::logging::{
    ensure_dir_exists, ensure_experiment_data_dir_exists, ensure_experiment_dir_exists,
    get_data_dir, get_experiment_log_dir, log_fitness_percentiles, log_status,
    logarithmic_tick_test, write_to_file,
};
use crate::biology::experiments::types::{GenomeExperimentEntry, TrialResultItem};
use crate::biology::genome::framed::render::with_stats::render_frames_with_stats;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use ron;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::gene_pool::{ExperimentGenePool, GenePoolId};
use super::types::MultiPoolExperimentSettings;

#[derive(Clone, Serialize, Deserialize)]
pub struct MultiPoolExperimentLoggingSettings {
    pub experiment_key: String,
    pub allow_overwrite: bool,
    pub checkpoint_interval: u64,
}

impl Default for MultiPoolExperimentLoggingSettings {
    fn default() -> Self {
        Self {
            experiment_key: "default".to_string(),
            allow_overwrite: true,
            checkpoint_interval: 10,
        }
    }
}

#[derive(Default)]
pub struct MultiPoolExperimentLogger {
    pub settings: MultiPoolExperimentLoggingSettings,
}

impl MultiPoolExperimentLogger {
    pub fn init(&self, gene_pools: &Vec<ExperimentGenePool>) {
        ensure_experiment_data_dir_exists();
        ensure_experiment_dir_exists(&self.settings.experiment_key, self.settings.allow_overwrite);

        let mut pools_path = get_experiment_log_dir(&self.settings.experiment_key);
        pools_path.push("gene_pools");
        ensure_dir_exists(&pools_path);

        for gene_pool in gene_pools {
            let gene_pool_dir_path = self.get_gene_pool_log_dir(gene_pool.id);
            println!("{}", gene_pool_dir_path.to_str().unwrap());
            if !gene_pool_dir_path.as_path().exists() {
                fs::create_dir(gene_pool_dir_path.as_path()).expect("failed to create path");
            }
        }

        self.init_reference_eval_results(gene_pools);
    }

    pub fn log_gene_pool_fitness_percentiles(&self, gene_pool: &ExperimentGenePool, tick: u64) {
        let mut path = self.get_gene_pool_log_dir(gene_pool.id);
        path.push("fitness.csv");

        log_fitness_percentiles(&path, tick, &gene_pool.state.genome_entries);
    }

    pub fn get_gene_pool_log_dir(&self, gene_pool_id: GenePoolId) -> PathBuf {
        let mut path = get_experiment_log_dir(&self.settings.experiment_key);

        path.push("gene_pools");
        path.push(format!("gene_pool_{}", gene_pool_id));

        path
    }

    /**
     * log csv where each row is an iteration and each column is a fitness score
     */
    pub fn log_gene_pool_summary(&self, gene_pool: &ExperimentGenePool) {
        let mut path = self.get_gene_pool_log_dir(gene_pool.id);
        path.push(format!("status-{}.txt", gene_pool.state.current_tick));
        log_status(
            &gene_pool.state.genome_entries,
            gene_pool.state.current_tick,
            &path,
            &gene_pool.gm,
        );
    }

    pub fn _get_reference_fitness_path(&self) -> PathBuf {
        let mut path = get_experiment_log_dir(&self.settings.experiment_key);
        path.push("reference_fitness.csv");

        path
    }

    pub fn init_reference_eval_results(&self, gene_pools: &Vec<ExperimentGenePool>) {
        let mut s = format!("tick,");

        for gene_pool in gene_pools.iter() {
            s.push_str(&format!("{},", &gene_pool.settings.name_key));
        }

        s.push_str("\n");

        let path = self._get_reference_fitness_path();
        write_to_file(path, s.as_bytes(), true);
    }

    pub fn log_reference_eval_results(&self, results: &Vec<TrialResultItem>, tick: u64) {
        let mut results = results.clone();
        results.sort_by_cached_key(|r| r.gene_pool_id);

        let mut s = format!("{},", tick);

        for (i, result) in results.iter().enumerate() {
            s.push_str(&format!("{},", result.fitness_score))
        }

        s.push_str("\n");

        let path = self._get_reference_fitness_path();
        write_to_file(path, s.as_bytes(), true);
    }
}

#[cfg(test)]
pub mod tests {}
