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
use std::time::Duration;

use super::logger::LoggingSettings;
use super::utils::{
    ExperimentGenomeUid, ExperimentSimSettings, GenomeEntryId, GenomeExperimentEntry,
    SimpleExperimentSettings,
};
use super::TrialResultItem;

const IS_LOGGING_ENABLED: bool = false;

macro_rules! explog {
    ($($arg:tt)*) => ({
		#[cfg(debug_assertions)]
		{
			if IS_LOGGING_ENABLED {println!($($arg)*)} else {}
		}
    })
}

pub struct SimRunnerGenomeEntry {
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
            let sim_unit_entry_id = entry.info.unit_entry_id;

            let genome_entry = &self.genomes[i];
            let genome_idx = genome_entry.genome_idx;
            let genome_uid = genome_entry.genome_uid;
            let genome = &genome_entry.genome;
            let stats = stat_entries[i].clone();

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
                .external_id(genome_entry.genome_idx)
                .build(&self.gm.chemistry_manifest);

            unit_entries.push(unit_entry);
            count += 1;
        }

        (unit_entries, stat_entries)
    }
}
