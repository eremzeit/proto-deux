use crate::biology::experiments::alterations;
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::render::with_stats::render_frames_with_stats;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::types::GenomeExperimentEntry;

const DATA_DIR_NAME: &str = "data";

pub fn get_data_dir() -> PathBuf {
    find_folder::Search::Parents(1)
        .for_folder(DATA_DIR_NAME)
        .expect("cant find data dir")
        .to_path_buf()
}

pub fn get_experiments_dir() -> PathBuf {
    let mut dir = get_data_dir();
    dir.push("experiments");
    dir
}

// used only for simple experiments
pub fn get_exp_genomes_dir(exp_key: &str) -> PathBuf {
    let mut dir = get_data_dir();
    dir.push("experiments");
    dir.push(exp_key);
    dir.push("genomes");
    dir
}

/**
 * Gets the path used for log files of this experiment
 */
pub fn get_experiment_log_dir(experiment_key: &str) -> PathBuf {
    let mut data_dir = get_data_dir().to_path_buf();

    // panic!("data dir: {}", data_dir.to_str().unwrap());
    data_dir.push("experiments");
    data_dir.push(experiment_key);

    // println!("LOG_DIR: {}", data_dir.to_str().unwrap());
    data_dir
}

pub fn ensure_experiment_data_dir_exists() {
    let mut experiments_path = get_data_dir().to_path_buf();
    experiments_path.push("experiments");

    if !experiments_path.as_path().exists() {
        fs::create_dir(experiments_path.as_path()).expect("failed to create path");
    }
}

pub fn ensure_dir_exists(path: &PathBuf) {
    let exists = path.exists();

    if !exists {
        fs::create_dir(path.as_path()).expect("failed to create path");
    }
}

pub fn ensure_experiment_dir_exists(key: &str, allow_overwrite: bool) {
    let log_path = get_experiment_log_dir(key);
    let exists = log_path.exists();
    //
    if exists {
        if !allow_overwrite {
            panic!("Experiment data dir {:?} aready exists", log_path);
        }

        println!("Removing path at: {}", log_path.to_str().unwrap());
        fs::remove_dir_all(log_path.as_path());
    }

    fs::create_dir(log_path.as_path()).expect("failed to create experiment log path");
}

pub fn write_to_file(file_path: PathBuf, buf: &[u8], append: bool) {
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

pub fn log_fitness_percentiles(
    path: &PathBuf,
    tick: u64,
    genome_entries: &Vec<GenomeExperimentEntry>,
) {
    let entries = genome_entries
        .iter()
        .filter(|e| e.max_fitness_metric.is_some())
        .collect::<Vec<_>>();

    let pcts = get_percentiles(entries.as_slice(), &[0, 25, 75, 100], |entry| {
        entry.max_fitness_metric.unwrap()
    });

    let mut s = format!("{},", tick);

    for i in 0..pcts.len() {
        s.push_str(format!("{}", pcts[i]).as_str());
        if i != pcts.len() - 1 {
            s.push_str(",");
        }
    }
    s.push_str("\n");

    write_to_file(path.clone(), s.as_bytes(), true);
}

fn get_percentiles<F, T, R>(items: &[T], percentiles: &[u8], f: F) -> Vec<R>
where
    F: Fn(&T) -> R,
    R: Ord + Copy,
{
    let mut values = items.iter().map(|i| f(i)).collect::<Vec<_>>();
    values.sort();

    if values.is_empty() {
        return vec![];
    }

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

pub fn log_status(
    genome_entries: &Vec<GenomeExperimentEntry>,
    tick: u64,
    path: &PathBuf,
    genetic_manifest: &GeneticManifest,
) {
    let mut s = String::new();
    let mut sorted_entries = genome_entries
        .iter()
        .filter(|e| e.max_fitness_metric.is_some())
        .collect::<Vec<_>>();
    sorted_entries.sort_by_key(|e| {
        e.max_fitness_metric
            .unwrap()
            .cmp(&(u64::MAX - e.max_fitness_metric.unwrap()))
    });

    for entry in sorted_entries.iter() {
        // let genome_str = FramedGenomeCompiler::compile(
        //     entry.compiled_genome.raw_values.clone(),
        //     genetic_manifest,
        // )
        // .display(genetic_manifest);
        let genome_str = render_frames_with_stats(
            &entry.compiled_genome.frames,
            genetic_manifest,
            Some(&entry.previous_execution_stats),
        );

        s.push_str(&format!(
            "------------------\n(uid: {}, fitness: {})\n",
            entry.uid,
            entry.max_fitness_metric.unwrap()
        ));
        s.push_str(&genome_str);
        s.push_str(&format!(
            "raw_genome: {:?}\n\n",
            &entry.compiled_genome.raw_values
        ));
        s.push_str(&format!(
            "raw_genome_length: {}\n",
            entry.compiled_genome.raw_size
        ));
        s.push_str(&format!("\n\n\n\n"));
    }

    write_to_file(path.clone(), s.as_bytes(), false);
}

#[cfg(test)]
pub mod tests {
    use crate::biology::experiments::{
        logging::logarithmic_tick_test, variants::simple::logger::SimpleExperimentLogger,
    };

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
    #[test]
    pub fn test_should_log_percentiles() {
        assert_eq!(logarithmic_tick_test(1), true);
        assert_eq!(logarithmic_tick_test(10), true);
        assert_eq!(logarithmic_tick_test(101), false);
        assert_eq!(logarithmic_tick_test(100), true);
        assert_eq!(logarithmic_tick_test(1001), false);
        assert_eq!(logarithmic_tick_test(1010), false);
        // assert_eq!(SimpleExperimentLogger::_should_log_percentiles(101), true);

        // for i in 0..10000000 {
        //     if SimpleExperimentLogger::_should_log_percentiles(i) {
        //         print!("tick: {}\n", i);
        //     }
        // }
        // panic!("");
    }
}

pub fn logarithmic_tick_test(tick: u64) -> bool {
    let factor = (10.0 as f32)
        .powf((tick as f32).log10().trunc() - 1.0)
        .max(1.0);
    tick % (factor as u64) == 0
}
