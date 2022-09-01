use ron;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    biology::unit_behavior::framed::FramedGenomeUnitBehavior, simulation::common::UnitManifest,
    unit_entry::builder::UnitEntryBuilder,
};

use crate::{
    biology::{
        experiments::variants::simple::utils::{
            get_exp_genomes_dir, get_experiments_dir, ExperimentSimSettings,
            SimpleExperimentSettings,
        },
        genetic_manifest::GeneticManifestData,
        genome::framed::{
            builders::FramedGenomeCompiler,
            common::{CompiledFramedGenome, FramedGenomeWord},
        },
    },
    chemistry::builder::ChemistryBuilder,
    simulation::common::{GeneticManifest, SimulationBuilder},
};
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

fn load_file(path: PathBuf) -> String {
    println!("path: {:?}", path);
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path.as_path())
        .unwrap();

    let mut str = String::new();
    file.read_to_string(&mut str);

    str
}

pub fn load_exp_settings(exp_key: &str) -> ExperimentSimSettings {
    let mut path = get_experiments_dir();
    path.push(exp_key);
    path.push("settings.ron");

    let s = load_file(path);

    let settings: ExperimentSimSettings = ron::from_str(&s).unwrap();
    settings
}

pub fn construct_replay_sim(exp_key: &str, genome_filename: &str) -> SimulationBuilder {
    let mut path = get_exp_genomes_dir(exp_key);
    path.push(genome_filename);

    let settings = load_exp_settings(exp_key);

    let chemistry_builder = settings.chemistry_options.clone();
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let genomes = load_genome_csv(path, &settings.chemistry_options.chemistry_key);
    let unit_entries = genomes
        .iter()
        .enumerate()
        .map(|(i, genome)| {
            UnitEntryBuilder::default()
                .species_name(format!("species{}", i))
                .behavior(FramedGenomeUnitBehavior::new(genome.clone(), gm.clone()).construct())
                .default_resources(settings.default_unit_resources.clone())
                .build(&chemistry_builder.manifest())
        })
        .collect::<Vec<_>>();

    SimulationBuilder::default()
        .chemistry(chemistry_builder.build())
        .size(settings.grid_size.clone())
        .iterations(settings.num_simulation_ticks)
        // .iterations(2)
        .unit_manifest(UnitManifest {
            units: unit_entries,
        })
}

pub fn load_genome_csv(path: PathBuf, chemistry_key: &str) -> Vec<Rc<CompiledFramedGenome>> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path.as_path())
        .unwrap();

    let mut str = String::new();
    file.read_to_string(&mut str);

    // println!("raw: {}", &str);
    let chemistry = ChemistryBuilder::with_key(chemistry_key).build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let genomes = str
        .split("\n")
        .filter(|s| !s.trim().is_empty())
        .map(|line| {
            let genome_vals = line
                .trim()
                .split(",")
                .map(|v| v.trim().parse::<FramedGenomeWord>().unwrap())
                .collect::<Vec<_>>();

            FramedGenomeCompiler::compile(genome_vals, &gm).wrap_rc()
        })
        .collect::<Vec<_>>();

    if genomes.len() == 0 {
        panic!("no genomes loaded");
    }

    // println!("{}", &genomes[0].display(&gm));
    println!("genome count: {}", &genomes.len());

    genomes
}

pub fn inspect_genome() {
    let mut path = find_folder::Search::Parents(1)
        .for_folder("data")
        .expect("cant find data dir")
        .to_path_buf();

    path.push("genomes");
    path.push("foo_genome.txt");

    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path.as_path())
        .unwrap();

    let mut str = String::new();
    file.read_to_string(&mut str);

    let vals = str
        .split(",")
        .map(|v| v.trim().parse::<FramedGenomeWord>().unwrap())
        .collect::<Vec<_>>();

    let chemistry = ChemistryBuilder::with_key("cheese").build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();
    let genome = FramedGenomeCompiler::compile(vals, &gm);
    println!("genome:\n{}", genome.display(&gm));
}
