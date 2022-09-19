use crate::biology::experiments::{alterations, fitness};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::annotated::FramedGenomeExecutionStats;
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
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use threadpool::ThreadPool;

use super::types::{ExperimentGenomeUid, ExperimentSimSettings, GenomeEntryId, TrialResultItem};
use super::variants::multi_pool::gene_pool::GenePoolId;

pub struct SimRunnerGenomeEntry {
    pub gene_pool_id: GenePoolId,
    pub genome_idx: GenomeEntryId,
    pub genome_uid: ExperimentGenomeUid,
    pub genome: CompiledFramedGenome,
    pub execution_stats: FramedGenomeExecutionStats,
}

pub struct ExperimentSimRunner {
    gm: Rc<GeneticManifest>,
    genomes: Vec<SimRunnerGenomeEntry>,
    sim_settings: ExperimentSimSettings,
    chemistry_builder: ChemistryBuilder,
    fitness_calculation_key: String,
}

impl ExperimentSimRunner {
    pub fn new(
        chemistry_builder: ChemistryBuilder,
        genomes: Vec<SimRunnerGenomeEntry>,
        sim_settings: ExperimentSimSettings,
        fitness_calculation_key: String,
    ) -> Self {
        let chemistry = chemistry_builder.clone().build();
        let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

        Self {
            genomes: genomes,
            chemistry_builder,
            gm,
            sim_settings,
            fitness_calculation_key,
        }
    }

    pub fn run_evaluation_for_uids(
        &mut self,
        // genome_uids: &Vec<ExperimentGenomeUid>,
    ) -> Vec<TrialResultItem> {
        let chemistry = self.chemistry_builder.build();

        perf_timer_start!("get_unit_entries");
        let (unit_entries, stat_entries) = self.get_unit_entries();
        perf_timer_stop!("get_unit_entries");
        // explog!("EVAL fitness for genomes: {:?}", genome_uids);
        let mut sim = SimulationBuilder::default()
            .size(self.sim_settings.grid_size.clone())
            .iterations(self.sim_settings.num_simulation_ticks)
            .chemistry(chemistry)
            .unit_manifest(UnitManifest {
                units: unit_entries,
            })
            .to_simulation();

        let mut executor = SimpleSimulationExecutor::new(sim);
        executor.start();

        let mut fitness_scores = vec![];
        let mut unit_entries = executor.simulation.unit_manifest.units.clone();
        unit_entries.sort_by_cached_key(|entry| entry.info.unit_entry_id);
        // println!(
        //     "unit_entries after sim: {:?}",
        //     unit_entries
        //         .iter()
        //         .map(|entry| entry.info.clone())
        //         .collect::<Vec<_>>()
        // );

        assert_eq!(unit_entries.len(), self.genomes.len());

        for i in (0..self.genomes.len()) {
            let entry = &unit_entries[i];
            let stats = stat_entries[i].clone();
            let sim_unit_entry_id = entry.info.unit_entry_id;

            let genome_entry = self
                .genomes
                .iter()
                .find(|genome_entry| genome_entry.genome_uid == entry.info.external_id)
                .unwrap();
            let genome_idx = genome_entry.genome_idx;
            let genome_uid = genome_entry.genome_uid;
            let genome = &genome_entry.genome;

            // let (genome_id, genome_uid, genome) = &self.genomes[i];
            // println!("{:?}", executor.simulation.unit_entry_attributes);

            let mut fitness_score = calculate_fitness(
                &self.fitness_calculation_key,
                entry.info.unit_entry_id,
                &mut executor.simulation.editable(),
            );

            // println!("fitness: {:?}", fitness_score);
            let penalty_pct = if genome.raw_size > 5000 {
                // (genome_entry.genome.len() as f64 / 4.0) as f64
                0.10
            } else {
                0.0
            };

            fitness_score = ((fitness_score as f64) * (1.0 - penalty_pct)) as u64;

            let resultItem = TrialResultItem {
                sim_unit_entry_id,
                experiment_genome_uid: genome_uid,
                fitness_score,
                genome_idx,
                stats: (*stats).clone().into_inner(),
            };
            fitness_scores.push(resultItem);
        }
        fitness_scores
    }

    pub fn get_unit_entries(
        &mut self,
    ) -> (Vec<UnitEntry>, Vec<Rc<RefCell<FramedGenomeExecutionStats>>>) {
        let mut unit_entries = vec![];
        let mut stat_entries = vec![];
        let mut count = 0;
        let cm = &self.gm.chemistry_manifest;
        for genome_entry in self.genomes.iter() {
            let stats = Rc::new(RefCell::new(genome_entry.execution_stats.clone()));
            stat_entries.push(stats.clone());

            let unit_entry = UnitEntryBuilder::default()
                .species_name(format!("species: {}", count))
                .behavior(
                    FramedGenomeUnitBehavior::new_with_stats(
                        Rc::new(genome_entry.genome.clone()),
                        Rc::new(self.gm.as_ref().clone()),
                        stats,
                    )
                    .construct(),
                )
                .default_resources(self.sim_settings.default_unit_resources.clone())
                .default_attributes(self.sim_settings.default_unit_attr.clone())
                .external_id(genome_entry.genome_uid)
                .build(&self.gm.chemistry_manifest);

            unit_entries.push(unit_entry);
            count += 1;
        }

        (unit_entries, stat_entries)
    }
}

pub fn execute_sim_runners(
    groups: Vec<Vec<SimRunnerGenomeEntry>>,
    use_threads: bool,
    sim_settings: &ExperimentSimSettings,
    fitness_calculation_key: &String,
) -> Vec<Vec<TrialResultItem>> {
    if use_threads {
        let (tx, rx) = mpsc::channel();
        let pool = ThreadPool::new(5);

        let group_count = groups.len();
        for entries in groups {
            let sim_settings = sim_settings.clone();
            let fitness_key = fitness_calculation_key.clone();
            let chemistry_builder = sim_settings.chemistry_options.clone();

            let tx = tx.clone();
            pool.execute(move || {
                let mut runner =
                    ExperimentSimRunner::new(chemistry_builder, entries, sim_settings, fitness_key);

                let result = runner.run_evaluation_for_uids();
                tx.send(result)
                    .expect("channel will be there waiting for the pool");
            });
        }
        rx.iter()
            .take(group_count)
            .collect::<Vec<Vec<TrialResultItem>>>()
    } else {
        let result = groups
            .into_iter()
            .map(|entries| {
                let sim_settings = sim_settings.clone();
                let fitness_key = fitness_calculation_key.clone();
                let chemistry_builder = sim_settings.chemistry_options.clone();

                let mut runner =
                    ExperimentSimRunner::new(chemistry_builder, entries, sim_settings, fitness_key);

                let result = runner.run_evaluation_for_uids();
                result
            })
            .collect::<Vec<_>>();

        result
    }
}
