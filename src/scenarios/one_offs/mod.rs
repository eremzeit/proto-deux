use crate::{
    biology::{
        genetic_manifest::GeneticManifest,
        genome::framed::{builders::FramedGenomeCompiler, common::FramedGenomeWord},
    },
    chemistry::builder::ChemistryBuilder,
};

pub mod cheese;
pub mod lever;
pub mod perf;

pub fn run_one_off(scenario_key: &str) {
    match scenario_key {
        "one_off_lever" => {
            lever::test_fitness(scenario_key);
        }
        "one_off_cheese" => {
            cheese::test_fitness(scenario_key);
        }
        "test_closure_perf" => {
            perf::closures::test_closure_perf();
        }
        "test_sim_perf" => {
            perf::sim::test_sim_perf();
        }
        _ => {
            panic!("Scenario key not found: {}", scenario_key);
        }
    };
}

pub fn inspect_genome() {
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;
    use std::rc::Rc;

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
    let gm = Rc::new(GeneticManifest::defaults(chemistry.get_manifest()));
    let genome = FramedGenomeCompiler::compile(vals, &gm);
    println!("genome:\n{}", genome.display(&gm));
}
