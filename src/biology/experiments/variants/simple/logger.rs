use crate::biology::experiments::alterations;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use std::fmt::{Debug, Formatter, Result};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::utils::{get_data_dir, GenomeExperimentEntry};

const DATA_DIR_NAME: &str = "data";

#[derive(Clone)]
pub struct LoggingSettings {
    pub experiment_key: String,
    pub allow_overwrite: bool,
    pub checkpoint_interval: usize,
}

impl Default for LoggingSettings {
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
    pub settings: LoggingSettings,
}

impl SimpleExperimentLogger {
    /**
     * Gets the path used for log files of this experiment
     */
    fn get_log_dir(&self) -> PathBuf {
        let mut data_dir = get_data_dir().to_path_buf();

        // panic!("data dir: {}", data_dir.to_str().unwrap());
        data_dir.push("experiments");
        data_dir.push(&self.settings.experiment_key);

        // println!("LOG_DIR: {}", data_dir.to_str().unwrap());
        data_dir
    }

    pub fn init(&self) {
        let mut experiments_path = get_data_dir().to_path_buf();
        experiments_path.push("experiments");

        if !experiments_path.as_path().exists() {
            fs::create_dir(experiments_path.as_path()).expect("failed to create path");
        }

        let log_path = self.get_log_dir();

        let exists = log_path.exists();

        //
        if exists {
            if !self.settings.allow_overwrite {
                panic!("Experiment data dir {:?} aready exists", log_path);
            }

            println!("Removing path at: {}", log_path.to_str().unwrap());
            fs::remove_dir_all(log_path.as_path());
        }

        fs::create_dir(log_path.as_path()).expect("failed to create experiment log path");
    }

    pub fn after_tick(
        &self,
        tick: usize,
        genome_entries: &Vec<GenomeExperimentEntry>,
        sensor_manifest: &SensorManifest,
        chemistry_manifest: &ChemistryManifest,
        genetic_manifest: &GeneticManifest,
    ) {
        if tick != 0 && tick % self.settings.checkpoint_interval == 0 {
            self._log_checkpoint(tick, genome_entries, genetic_manifest);
        }

        self._log_fitness_percentiles(tick, genome_entries);
    }

    fn _write_to_file(&self, file_path: PathBuf, buf: &[u8], append: bool) {
        // println!("append: {}", append);
        // println!("file path: {}", file_path.as_path().to_str().unwrap());
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(append) // This is needed to append to file
            .open(file_path.as_path())
            .unwrap();

        file.write_all(buf);
    }

    // log csv where each row is an iteration and each column is a fitness score
    pub fn _log_fitness_percentiles(
        &self,
        tick: usize,
        genome_entries: &Vec<GenomeExperimentEntry>,
    ) {
        let mut path = self.get_log_dir();
        path.push("fitness.csv");

        let entries = genome_entries
            .iter()
            .filter(|e| e.max_fitness_metric.is_some())
            .collect::<Vec<_>>();

        let pcts = get_percentiles(entries.as_slice(), &[0, 25, 75, 100], |entry| {
            entry.max_fitness_metric.unwrap()
        });

        let mut s = format!("({}),", tick);

        for i in 0..pcts.len() {
            s.push_str(format!("{}", pcts[i]).as_str());
            if i != pcts.len() - 1 {
                s.push_str(",");
            }
        }
        s.push_str("\n");

        self._write_to_file(path, s.as_bytes(), true)
    }

    pub fn _log_checkpoint(
        &self,
        tick: usize,
        genome_entries: &Vec<GenomeExperimentEntry>,

        genetic_manifest: &GeneticManifest,
    ) {
        let mut s = String::new();
        let mut sorted_entries = genome_entries
            .iter()
            .filter(|e| e.max_fitness_metric.is_some())
            .collect::<Vec<_>>();
        sorted_entries.sort_by(|e1, e2| {
            e1.max_fitness_metric
                .unwrap()
                .cmp(&e2.max_fitness_metric.unwrap())
        });

        for entry in sorted_entries.iter() {
            let genome_str = FramedGenomeCompiler::compile(entry.genome.clone(), genetic_manifest)
                .display(genetic_manifest);

            s.push_str(&format!(
                "{}------------------",
                entry.max_fitness_metric.unwrap()
            ));
            s.push_str(&genome_str);
            s.push_str(&format!("raw_genome: {:?}/n", entry.genome.clone()));
            s.push_str(&format!(
                "raw_genome_length: {}/n",
                entry.genome.clone().len()
            ));
            s.push_str(&format!("\n\n\n\n"));
        }

        let mut path = self.get_log_dir();
        path.push(format!("checkpoint-{}.csv", tick));
        self._write_to_file(path, s.as_bytes(), false);
    }
}

fn get_percentiles<F, T, R>(items: &[T], percentiles: &[u8], f: F) -> Vec<R>
where
    F: Fn(&T) -> R,
    R: Ord + Copy,
{
    let mut values = items.iter().map(|i| f(i)).collect::<Vec<_>>();
    values.sort();

    let mut result: Vec<R> = vec![];
    for p in percentiles.iter() {
        if *p > 100 {
            panic!("percentile out of range");
        }

        let index = ((items.len() - 1) * (*p as usize)) / 100;

        result.push(values[index]);
    }

    result
}
pub mod tests {
    use super::get_percentiles;

    #[test]
    pub fn test_percentiles() {
        let mut values = vec![];
        for i in 0..100 {
            values.push(i);
        }

        let _vals = values.as_slice();
        let pcts = get_percentiles(_vals, &[0, 50, 100], |x| *x);

        assert_eq!(pcts.len(), 3);
        assert_eq!(pcts[0], 0);
        assert_eq!(pcts[1], 49);
        assert_eq!(pcts[2], 99);
    }
}
