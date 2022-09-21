use crate::biology::experiments::alterations;
use crate::biology::experiments::logging::{
    ensure_experiment_data_dir_exists, ensure_experiment_dir_exists, get_data_dir,
    get_experiment_log_dir, log_fitness_percentiles, log_status, logarithmic_tick_test,
    write_to_file,
};
use crate::biology::experiments::types::GenomeExperimentEntry;
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

use super::utils::SimpleExperimentSettings;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimpleExperimentLoggingSettings {
    pub experiment_key: String,
    pub allow_overwrite: bool,
    pub checkpoint_interval: u64,
}

impl Default for SimpleExperimentLoggingSettings {
    fn default() -> Self {
        Self {
            experiment_key: "default".to_string(),
            allow_overwrite: true,
            checkpoint_interval: 10,
        }
    }
}

#[derive(Default)]
pub struct SimpleExperimentLogger {
    pub settings: SimpleExperimentLoggingSettings,
}

impl SimpleExperimentLogger {
    pub fn init(&self) {
        ensure_experiment_data_dir_exists();
        ensure_experiment_dir_exists(&self.settings.experiment_key, self.settings.allow_overwrite);

        let mut genome_path = get_experiment_log_dir(&self.settings.experiment_key);
        genome_path.push("genomes");
        if !genome_path.exists() {
            fs::create_dir(genome_path.as_path()).expect("failed to create genome log path");
        }
    }

    pub fn log_settings(&self, settings: &SimpleExperimentSettings) {
        let mut path = get_experiment_log_dir(&self.settings.experiment_key);
        path.push("settings.ron");

        let settings_str = ron::to_string(&settings.sim_settings).unwrap();
        write_to_file(path, settings_str.as_bytes(), false);
    }

    pub fn log_best_genomes(
        &self,
        tick: u64,
        genome_entries: &Vec<GenomeExperimentEntry>,
        num_genomes: usize,
    ) {
        if !logarithmic_tick_test(tick) {
            return;
        }
        let mut entries = genome_entries.clone();

        entries.sort_by_key(|entry| entry.max_fitness_metric.unwrap_or(0));
        entries.reverse();

        if entries.len() == 0 {
            print!("RETURNING EARLY");
            return;
        }

        let entries = entries.iter().take(num_genomes);

        let mut path = get_experiment_log_dir(&self.settings.experiment_key);
        path.push("genomes");
        path.push(format!("{}.csv", tick));

        let mut s = String::new();

        for entry in entries {
            let genome = &entry.compiled_genome.raw_values;
            for i in 0..genome.len() {
                if i == genome.len() - 1 {
                    s.push_str(&format!("{}", &genome[i]));
                } else {
                    s.push_str(&format!("{},", &genome[i]));
                }
            }

            s.push_str("\n");
        }

        // s.push_str(&format!("raw_genome: {:?}\n\n", entry.genome.clone()));
        write_to_file(path, s.as_bytes(), false);
    }

    pub fn log_fitness_percentiles(
        &self,
        tick: u64,
        genome_entries: &Vec<GenomeExperimentEntry>,
        max_ticks: u64,
    ) {
        let interval = (max_ticks / 1000).max(10);

        if tick % interval != 0 {
            return;
        }

        let mut path = get_experiment_log_dir(&self.settings.experiment_key);
        path.push("fitness.csv");

        log_fitness_percentiles(&path, tick, genome_entries);
    }

    /**
     * log csv where each row is an iteration and each column is a fitness score
     */
    pub fn log_status(
        &self,
        tick: u64,
        genome_entries: &Vec<GenomeExperimentEntry>,
        genetic_manifest: &GeneticManifest,
    ) {
        let mut path = get_experiment_log_dir(&self.settings.experiment_key);
        path.push(format!("status-{}.txt", tick));
        log_status(genome_entries, tick, &path, genetic_manifest);
    }
}

#[cfg(test)]
pub mod tests {}
