pub mod logger;
pub mod serialize;
pub mod utils;

use crate::biology::experiments::alterations;
use crate::biology::experiments::variants::simple::utils::{
    ExperimentGenomeUid, GenomeExperimentEntry,
};
use crate::biology::unit_behavior::framed::common::*;
use crate::perf::{perf_timer_start, perf_timer_stop};
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use std::fmt::{Debug, Formatter, Result};
use std::ops::{Add, Div};
use std::time::Duration;

use self::logger::SimpleExperimentLogger;
use self::utils::SimpleExperimentSettings;
use crate::biology::genome::framed::samples;

macro_rules! is_experiment_logging_enabled {
    ( ) => {
        // true
        false
    };
}

#[macro_export]
macro_rules! explog {
    ($($arg:tt)*) => ({
        if is_experiment_logging_enabled!() {println!($($arg)*)} else {}
    })
}

pub struct SimpleExperiment {
    pub is_paused: bool,
    pub is_initialized: bool,
    pub genome_entries: Vec<GenomeExperimentEntry>,
    pub is_headless: bool,
    pub current_tick: u64,

    pub settings: SimpleExperimentSettings,
    pub _last_entry_id: usize,

    _logger: Option<SimpleExperimentLogger>,
    _seed_genomes: Option<Vec<RawFramedGenome>>,
}

impl SimpleExperiment {
    pub fn new(settings: SimpleExperimentSettings) -> Self {
        let logger = settings
            .logging_settings
            .as_ref()
            .map(|settings| SimpleExperimentLogger {
                settings: settings.clone(),
            });

        SimpleExperiment {
            current_tick: 0,
            is_paused: true,
            is_initialized: false,
            genome_entries: vec![],
            is_headless: true,
            settings,
            _last_entry_id: 0,

            _logger: logger,
            _seed_genomes: None,
        }
    }

    pub fn initialize(&mut self) {
        if let Some(logger) = &self._logger {
            logger.init();
        }

        if self.settings.num_genomes < self.settings.sim_settings.num_genomes_per_sim {
            panic!(
                "Number of genomes in pool must be larger than genomes in a single sim: ({}, {})",
                self.settings.num_genomes, self.settings.sim_settings.num_genomes_per_sim
            );
        }

        self.populate_initial_genomes();

        self.is_initialized = true;
    }

    pub fn with_seed_genomes(&mut self, genomes: Vec<RawFramedGenome>) {
        if genomes.len() > self.settings.num_genomes {
            panic!("Too many genomes given");
        }

        self._seed_genomes = Some(genomes);
    }

    pub fn populate_initial_genomes(&mut self) {
        if let Some(seed_genomes) = self._seed_genomes.clone() {
            for genome in seed_genomes.into_iter() {
                self.register_new_genome(genome);
            }
        }

        let still_need = self.settings.num_genomes - self.genome_entries.len();

        let mut rng = rand::thread_rng();

        for i in (0..still_need) {
            self.register_new_genome(random_genome_of_length(rng.gen_range((30..50))));
        }
    }

    pub fn register_new_genome(&mut self, genome: RawFramedGenome) {
        let next_genome_id = if self.genome_entries.len() > 0 {
            self._last_entry_id + 1
        } else {
            0
        };

        let genome_entry = GenomeExperimentEntry {
            last_fitness_metrics: vec![],
            max_fitness_metric: None,
            num_evaluations: 0,
            genome: genome,
            uid: next_genome_id as ExperimentGenomeUid,
            current_rank_score: 0,
        };

        self._last_entry_id = genome_entry.uid;

        self.genome_entries.push(genome_entry);
    }

    pub fn start(&mut self) {
        if !self.is_initialized {
            self.initialize();
        }
        self.resume();
    }

    pub fn resume(&mut self) {
        if !self.is_initialized {
            panic!("Experiment hasn't been initialized");
        }

        if let Some(logger) = &self._logger  && self.current_tick == 0{
            logger._log_status(
                self.current_tick as usize,
                &self.genome_entries,
                &self.settings.gm,
            );
        }

        while self.current_tick < self.settings.iterations as u64 {
            perf_timer_start!("experiment_tick");
            self.tick();
            perf_timer_stop!("experiment_tick");
            self.current_tick += 1;
        }
        explog!("FINAL STATUS: {:?}", &self.__gather_status());
    }

    pub fn _find_newest_genome_uid(&self) -> ExperimentGenomeUid {
        let mut min_i = 0;
        let mut min_evals = None;
        for (i, genome) in self.genome_entries.iter().enumerate() {
            if min_evals.is_none() || genome.num_evaluations < min_evals.clone().unwrap() {
                min_evals = Some(genome.num_evaluations);
                min_i = i;
            }
        }

        self.genome_entries[min_i].uid
    }

    pub fn partition_into_groups(&mut self) -> Vec<Vec<ExperimentGenomeUid>> {
        // explog!(
        //     "partitioning uids: {:?}",
        //     self.genome_entries
        //         .iter()
        //         .map(|unit| { unit.uid })
        //         .collect::<Vec<_>>()
        // );
        let num_genomes = self.genome_entries.len();
        let group_size = self.settings.sim_settings.num_genomes_per_sim;

        let mut entries_by_fitness = self.genome_entries.clone();
        entries_by_fitness.sort_by_key(|entry| entry.current_rank_score);

        let mut partitions = vec![];
        let mut num_groups = num_genomes / group_size;

        let mut last_group_is_uneven = false;
        if num_genomes % group_size > 0 {
            num_groups += 1;
            last_group_is_uneven = true;
        }

        // println!(
        //     "uids in entries_by_fitness: {:?}",
        //     entries_by_fitness
        //         .iter()
        //         .map(|unit| { unit.uid })
        //         .collect::<Vec<_>>()
        // );

        let mut genome_idx = 0;
        for group_idx in (0..num_groups) {
            let mut group = vec![];

            if last_group_is_uneven && group_idx == num_groups - 1 {
                for i in (0..group_size) {
                    // count backwards from the end of the genome list. ie. some genomes will get included in two groups
                    let genome_idx = num_genomes - 1 - i;
                    group.insert(0, entries_by_fitness[genome_idx].uid);
                    // group.push(entries_by_fitness[genome_idx].uid);
                }
            } else {
                for i in (0..group_size) {
                    group.push(entries_by_fitness[genome_idx].uid);
                    genome_idx += 1;
                }
            }

            partitions.push(group);
        }
        //let flattened = partitions.iter().flatten().collect::<Vec<_>>();
        //println!("flattened groups: {:?}", flattened.len());
        partitions
    }

    pub fn _summarize_genomes(&self, gm: &GeneticManifest) -> String {
        let mut s = "".to_string();

        self.genome_entries.iter().map(|entry| {
            let genome_vals = entry.genome.clone();
            let genome = FramedGenomeCompiler::compile(genome_vals, gm);
            s = format!("{}\n{}", s, genome.display(&self.settings.gm));
        });
        s
    }

    pub fn _get_unit_entries_for_uids(
        &self,
        uids: &[ExperimentGenomeUid],
        cm: &ChemistryManifest,
        // chemistry: &ChemistryInstance,
    ) -> Vec<UnitEntry> {
        let mut unit_entries = vec![];
        let mut count = 0;
        for uid in uids {
            perf_timer_start!("building_unit_entries");
            let maybe_idx = self._find_by_uid(*uid);
            let idx = maybe_idx.unwrap();

            let genome = self.genome_entries[idx].compile(&self.settings.gm);

            let libraries =
                construct_chemistry_libraries(&self.settings.chemistry_options.chemistry_key);

            let gm = self.settings.gm.clone();
            // let gm = Rc::new(self.settings.gm.to_compiled(
            //     libraries.action_library,
            //     libraries.custom_sensor_library,
            //     libraries.operator_library,
            // ));
            let unit_entry = UnitEntryBuilder::default()
                .species_name(format!("species: {}", count))
                .behavior(FramedGenomeUnitBehavior::new(genome, gm.clone()).construct())
                .default_resources(self.settings.sim_settings.default_unit_resources.clone())
                .default_attributes(self.settings.sim_settings.default_unit_attr.clone())
                .build(cm);
            perf_timer_stop!("building_unit_entries");

            unit_entries.push(unit_entry);
            count += 1;
        }

        unit_entries
    }

    pub fn run_evaluation_for_uids(
        &mut self,
        genome_uids: &Vec<ExperimentGenomeUid>,
    ) -> Vec<TrialResultItem> {
        let chemistry = self.settings.chemistry_options.build();

        perf_timer_start!("get_unit_entries");
        let unit_entries = self._get_unit_entries_for_uids(
            genome_uids.as_slice(),
            &self.settings.gm.chemistry_manifest,
            // &chemistry,
        );

        perf_timer_stop!("get_unit_entries");

        explog!("EVAL fitness for genomes: {:?}", genome_uids);

        perf_timer_start!("sim_build");
        let mut sim = SimulationBuilder::default()
            .size(self.settings.sim_settings.grid_size.clone())
            .iterations(self.settings.sim_settings.num_simulation_ticks)
            .chemistry(chemistry)
            .unit_manifest(UnitManifest {
                units: unit_entries,
            })
            .to_simulation();
        perf_timer_stop!("sim_build");

        perf_timer_start!("sim_eval");
        let mut executor = SimpleSimulationExecutor::new(sim);
        executor.start();
        perf_timer_stop!("sim_eval");

        // println!(
        //     "unit_entry_attributes: {:?}",
        //     executor.simulation.unit_entry_attributes
        // );

        // panic!("AOEU"); // unit_entry_attributes arent' being calculated

        perf_timer_start!("experiment_fitness_tally");
        let mut fitness_scores = vec![];
        let mut unit_entries = executor.simulation.unit_manifest.units.clone();
        unit_entries.sort_by_key(|entry| entry.info.id);
        // println!(
        //     "unit_entries after sim: {:?}",
        //     unit_entries
        //         .iter()
        //         .map(|entry| entry.info.clone())
        //         .collect::<Vec<_>>()
        // );

        assert_eq!(unit_entries.len(), genome_uids.len());

        for (i, entry) in unit_entries.iter().enumerate() {}

        for i in (0..genome_uids.len()) {
            let entry = &unit_entries[i];
            let sim_unit_entry_id = entry.info.id;
            let genome_uid = genome_uids[i as usize];
            let genome_idx = self._find_by_uid(genome_uid).unwrap();

            let mut genome_entry = &mut self.genome_entries[genome_idx];
            genome_entry.num_evaluations += 1;

            // println!("fitness key: {}", self.settings.fitness_calculation_key);
            let mut fitness_score = calculate_fitness(
                &self.settings.fitness_calculation_key,
                entry.info.id,
                &mut executor.simulation.editable(),
            );

            let penalty = if genome_entry.genome.len() > 1000 {
                (genome_entry.genome.len() / 4) as u64
            } else {
                0 as u64
            };

            if penalty < fitness_score {
                fitness_score -= penalty;
            } else {
                fitness_score = 0;
            }

            let resultItem = TrialResultItem {
                sim_unit_entry_id,
                experiment_genome_uid: genome_uid,
                genome_idx,
                fitness_score,
            };
            fitness_scores.push(resultItem);
        }
        perf_timer_stop!("experiment_fitness_tally");

        fitness_scores
    }

    pub fn normalize_ranks(&mut self) {
        // let highest_rank = self.genome_entries.len() - 1;

        // let mut entries = self.genome_entries.clone();
        // entries.sort_by_key(|entry| entry.current_rank_score);

        //normalize
        let mut sorted_ranks = self
            .genome_entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (entry.uid, entry.current_rank_score))
            .collect::<Vec<_>>();
        sorted_ranks.sort_by_key(|(genome_id, score)| *score);

        // explog!("normalizing: {:?}", &sorted_ranks);

        let mut current_rank_tally = 0;
        let mut prev_rank = None;
        for i in (0..sorted_ranks.len()) {
            let _current = sorted_ranks[i].1;
            if prev_rank != None && prev_rank != Some(_current) {
                current_rank_tally += 1;
            }
            // explog!(
            //     "[i:{}] rank_tally: {:?} -- current_rank: {} -- prev_rank: {:?}",
            //     i,
            //     current_rank_tally,
            //     _current,
            //     prev_rank
            // );
            sorted_ranks[i].1 = current_rank_tally;

            prev_rank = Some(_current);
        }

        for i in (0..sorted_ranks.len()) {
            let uid = sorted_ranks[i].0;
            let rank = sorted_ranks[i].1;
            let index = self._find_by_uid(uid).unwrap();
            self.genome_entries[index].current_rank_score = rank;
        }
    }

    pub fn update_genomes_with_fitness_result(&mut self, fitness_result: &Vec<TrialResultItem>) {
        let mut sorted_result = fitness_result.clone();
        sorted_result.sort_by_key(|x| x.fitness_score);

        for (result_rank, fitness_result_item) in sorted_result.iter().enumerate() {
            let our_genome_idx = fitness_result_item.genome_idx;
            let our_genome_uid = fitness_result_item.experiment_genome_uid;
            let our_fitness_score = fitness_result_item.fitness_score;

            let mut max_fitness = self.genome_entries[our_genome_idx]
                .max_fitness_metric
                .unwrap_or(0);

            let mut is_new_max = false;
            if our_fitness_score > max_fitness {
                max_fitness = our_fitness_score;
                is_new_max = true;
            }

            self.genome_entries[our_genome_idx].max_fitness_metric = Some(max_fitness);

            push_into_with_max(
                &mut self.genome_entries[our_genome_idx].last_fitness_metrics,
                our_fitness_score,
                10,
            );
            // println!(
            //     "processing fitness for id: {} with score {}",
            //     genome_id, self.genome_entries[*genome_id as usize].current_rank_score
            // );

            let our_rank_score = self.genome_entries[our_genome_idx].current_rank_score;

            for i in (0..sorted_result.len()) {
                let their_uid = sorted_result[i].experiment_genome_uid;
                let their_genome_idx = sorted_result[i].genome_idx;

                let their_rank_score = self.genome_entries[their_genome_idx].current_rank_score;
                let their_fitness_score = sorted_result[i].fitness_score;

                let mut our_new_rank_score = self.genome_entries[our_genome_idx].current_rank_score;
                let is_upset =
                    our_fitness_score > their_fitness_score && our_rank_score <= their_rank_score;
                if is_upset {
                    our_new_rank_score = their_rank_score + 1;
                    explog!(
                        "\t-> {} (r:{}, {}) upsets against {} (r:{}, {}) -- new rank: {}",
                        our_genome_uid,
                        our_rank_score,
                        our_fitness_score,
                        their_uid,
                        their_rank_score,
                        their_fitness_score,
                        our_new_rank_score
                    );
                }
                self.genome_entries[our_genome_idx].current_rank_score = our_new_rank_score;
            }
        }
        self.normalize_ranks();
    }

    pub fn cull_and_replace(&mut self) {
        let target_count = (self.settings.num_genomes as f64 * 0.50) as usize;
        let to_remove = self.genome_entries.len() - target_count;
        let mut by_rank = self.genome_entries.clone();
        by_rank.sort_by_key(|entry| entry.current_rank_score);
        // by_rank.reverse();

        // println!("Current genome ranks...");
        // for genome in by_rank.iter() {
        //     println!(
        //         "{:?}",
        //         (
        //             genome.uid,
        //             genome.current_rank_score,
        //             genome.max_fitness_metric,
        //             calculate_mean(&genome.last_fitness_metrics)
        //         )
        //     );
        // }

        let mut removed_count = 0;
        let mut uids_to_remove = vec![];

        // TODO: this might be incorrect
        let mut offset_from_bottom = 0;
        while removed_count < to_remove && offset_from_bottom < self.genome_entries.len() {
            while offset_from_bottom < self.genome_entries.len()
                && by_rank[offset_from_bottom].max_fitness_metric.is_none()
            {
                offset_from_bottom += 1;
            }
            let item = by_rank.remove(offset_from_bottom);
            // println!(
            //     "Removing genome: {:?}",
            //     (
            //         item.uid,
            //         item.current_rank_score,
            //         item.max_fitness_metric,
            //         calculate_mean(&item.last_fitness_metrics)
            //     )
            // );

            uids_to_remove.push(item.uid);
            removed_count += 1;
        }

        explog!("Culling genomes with uids: {:?}", &uids_to_remove);
        for uid in uids_to_remove {
            let idx = self._find_by_uid(uid).unwrap();
            self.genome_entries.remove(idx);
        }

        while self.genome_entries.len() < self.settings.num_genomes {
            self.pull_fresh_genome();
        }
    }

    pub fn pull_fresh_genome(&mut self) {
        let mut rng = rand::thread_rng();
        let alt_i = rng.gen_range((0..self.settings.alteration_set.alterations.len()));
        self._pull_fresh_genome(alt_i);
    }

    fn _pull_fresh_genome(&mut self, alteration_index: usize) {
        //explog!("Pulling new genome...");
        let alteration = &self.settings.alteration_set.alterations[alteration_index].clone();

        let mut genomes = vec![];
        for i in (0..alteration.genomes_required) {
            let uid = self._select_random_top_genome();
            let index = self._find_by_uid(uid).unwrap();
            genomes.push(self.genome_entries[index].genome.clone());
        }

        let params = (alteration.prepare)(&genomes.as_slice());
        let new_genome = (alteration.execute)(&genomes.as_slice(), &params.as_slice());

        if new_genome.len() > 0 {
            self.register_new_genome(new_genome);
        }
    }

    fn _select_random_top_genome(&self) -> usize {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range((0..self.genome_entries.len()));
        let mut by_rank = self.genome_entries.clone();
        by_rank.sort_by_key(|entry| entry.current_rank_score);
        by_rank[i].uid
    }
    fn _find_by_uid(&self, uid: usize) -> Option<usize> {
        for i in (0..self.genome_entries.len()) {
            if self.genome_entries[i].uid == uid {
                return Some(i);
            };
        }

        None
    }

    fn __gather_status(&self) -> Vec<[usize; 4]> {
        let mut statuses = self
            .genome_entries
            .iter()
            .map(|x| {
                [
                    x.uid,
                    x.max_fitness_metric.unwrap_or(0) as usize,
                    x.num_evaluations,
                    x.current_rank_score,
                ]
            })
            .collect::<Vec<_>>();

        statuses.sort_by_key(|x| x[2]);
        statuses.reverse();
        statuses
    }

    fn get_max_fitness_for_genome(&self, uid: ExperimentGenomeUid) -> Option<FitnessScore> {
        let idx = self._find_by_uid(uid).unwrap();
        self.genome_entries[idx].max_fitness_metric
    }

    fn get_mean_fitness_for_genome(&self, uid: ExperimentGenomeUid) -> Option<FitnessScore> {
        let idx = self._find_by_uid(uid).unwrap();
        calculate_mean(&self.genome_entries[idx].last_fitness_metrics)
    }

    fn _highest_fitness_idx(&self) -> usize {
        let uid = self
            .genome_entries
            .iter()
            .max_by_key(|entry| self.get_mean_fitness_for_genome(entry.uid).unwrap_or(0))
            .unwrap()
            .uid
            .clone();
        self._find_by_uid(uid).unwrap()
    }

    pub fn tick(&mut self) {
        perf_timer_start!("experiment_partition");
        let groups = self.partition_into_groups();
        explog!("groups: {:?}", &groups);
        perf_timer_stop!("experiment_partition");

        perf_timer_start!("experiment_sim_eval");
        for group in groups {
            // std::thread::sleep(Duration::from_millis(10));
            let fitness_result = self.run_evaluation_for_uids(&group);
            // println!("fitness_scores: {:?}", fitness_result);
            perf_timer_start!("adjust_ranks");
            self.update_genomes_with_fitness_result(&fitness_result);
            perf_timer_stop!("adjust_ranks");
        }
        perf_timer_stop!("experiment_sim_eval");

        explog!("status after adjusting: {:?}", &self.__gather_status());
        perf_timer_start!("experiment_cull_and_replace");
        self.cull_and_replace();
        perf_timer_stop!("experiment_cull_and_replace");
        explog!("status after culling: {:?}", &self.__gather_status());

        let mut max_scores = self
            .genome_entries
            .iter()
            .map(|entry| (entry.uid, entry.max_fitness_metric))
            .collect::<Vec<_>>();
        max_scores.sort_by_key(|entry| entry.1);
        max_scores.reverse();

        // let (cm, sm, gm) = self.settings.specs.context();
        if let Some(logger) = &self._logger {
            perf_timer_start!("experiment_logging");
            logger._log_fitness_percentiles(self.current_tick as usize, &self.genome_entries);

            let _tick = self.current_tick + 1;
            let should_log_checkpoint = _tick % logger.settings.checkpoint_interval as u64 == 0
                || _tick >= self.settings.iterations as u64
                || self.current_tick == 1;
            if should_log_checkpoint {
                logger._log_status(_tick as usize, &self.genome_entries, &self.settings.gm)
            }
            perf_timer_stop!("experiment_logging");
        }

        if self.current_tick % 100 == 0 {
            println!(
                "EXPERIMENT TICK: {} -- fitness: {:?}",
                self.current_tick,
                self.genome_entries[self._highest_fitness_idx()].last_fitness_metrics
            );
        }
    }
}

pub fn calculate_mean(vec: &Vec<FitnessScore>) -> Option<FitnessScore> {
    if vec.len() == 0 {
        return None;
    }

    let mut ave = vec[0];

    let mut i = 1;
    while i < vec.len() {
        ave = ave + vec[i];
        i += 1;
    }

    Some(ave / vec.len() as FitnessScore)
}

pub fn push_into_with_max<T>(vec: &mut Vec<T>, item: T, max_size: usize) {
    vec.insert(0, item);

    while vec.len() > max_size {
        vec.pop();
    }
}

pub fn random_genome_of_length(length: usize) -> Vec<FramedGenomeWord> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vals = vec![];

    for i in (0..length) {
        vals.push(rng.gen::<FramedGenomeWord>());
    }

    vals
}

pub mod tests {
    use variants::CheeseChemistry;

    use self::utils::SimpleExperimentSettingsBuilder;
    use super::*;
    use crate::biology::experiments::variants::simple::logger::LoggingSettings;
    use crate::biology::experiments::variants::simple::utils::{
        CullStrategy, ExperimentSimSettings,
    };
    use crate::biology::genetic_manifest::GeneticManifest;
    use crate::biology::genome::framed::common::*;
    use crate::biology::unit_behavior::framed::common::*;
    use crate::simulation::common::builder::ChemistryBuilder;
    use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
    use crate::simulation::common::*;

    pub fn experiment_settings() -> SimpleExperimentSettingsBuilder {
        let chemistry_builder = ChemistryBuilder::with_key("cheese");
        let chemistry = chemistry_builder.build();

        // let gm = GeneticManifest::from_chemistry_with_defaults::<CheeseChemistry>().wrap_rc();
        let gm =
            GeneticManifest::construct::<CheeseChemistry>(&chemistry.get_configuration()).wrap_rc();

        SimpleExperimentSettingsBuilder::default()
            .alteration_set(alterations::default_alteration_set())
            .experiment_key("my_experiment".to_string())
            .cull_strategy(CullStrategy::WorstFirst)
            .fitness_calculation_key("total_cheese_consumed".to_string())
            .num_genomes(11)
            .sim_settings(ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                default_unit_resources: vec![("cheese".to_owned(), 200)],
                default_unit_attr: vec![],
                place_units_method: PlaceUnitsMethod::Default,
            })
            .iterations(1)
            .logging_settings(None)
            .chemistry_options(chemistry_builder)
            .gm(gm)
    }

    #[test]
    fn experiment_initialization() {
        let settings = experiment_settings().build().unwrap();
        let mut exp = SimpleExperiment::new(settings);
        exp.initialize();

        assert_eq!(exp.genome_entries.len(), 11);
    }

    #[test]
    fn test_partition_into_groups() {
        let num_genomes = 5;
        let mut settings = experiment_settings()
            .num_genomes(num_genomes)
            .build()
            .unwrap();
        let mut exp = SimpleExperiment::new(settings);
        exp.initialize();

        // modify the genome list to be in decreasing order of fitness rank
        for i in 0..num_genomes {
            exp.genome_entries[i].current_rank_score = num_genomes - i;
        }

        let groups = exp.partition_into_groups();

        // println!("groups: {:?}", groups);

        let flattened = groups.iter().flatten().map(|x| *x).collect::<Vec<_>>();
        assert_eq!(flattened.len(), num_genomes + 1); //because the genome count isn't a multiple of the group size, there's double inclusion
        assert_eq!(flattened, vec![4, 3, 2, 1, 1, 0]);
        assert_eq!(groups, vec![vec![4, 3], vec![2, 1], vec![1, 0]]);
    }

    #[test]
    fn test_random_genome_generation() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>().wrap_rc();

        let vals1 = random_genome_of_length(100);

        assert_eq!(vals1.len(), 100);
        let genome1 = FramedGenomeCompiler::compile(vals1, &gm);
        print!("random genome: {}\n", genome1.display(&gm));
    }

    #[test]
    fn test_adjust_rank() {
        let num_genomes = 4;
        let mut settings = experiment_settings()
            .num_genomes(num_genomes)
            .build()
            .unwrap();

        let mut exp = SimpleExperiment::new(settings);
        exp.populate_initial_genomes();
        exp.genome_entries[0].current_rank_score = 0;
        exp.genome_entries[0].max_fitness_metric = Some(175);
        exp.genome_entries[1].current_rank_score = 1;
        exp.genome_entries[1].max_fitness_metric = Some(300);
        exp.genome_entries[2].current_rank_score = 2;
        exp.genome_entries[2].max_fitness_metric = Some(400);
        exp.genome_entries[3].current_rank_score = 3;
        exp.genome_entries[3].max_fitness_metric = Some(800);

        let fitness_result = vec![
            TrialResultItem {
                genome_idx: 0,
                sim_unit_entry_id: 0,
                experiment_genome_uid: 0,
                fitness_score: 200,
            },
            TrialResultItem {
                genome_idx: 1,
                sim_unit_entry_id: 1,
                experiment_genome_uid: 1,
                fitness_score: 125,
            },
            TrialResultItem {
                genome_idx: 2,
                sim_unit_entry_id: 2,
                experiment_genome_uid: 2,
                fitness_score: 100,
            },
        ];

        exp.update_genomes_with_fitness_result(&fitness_result);

        assert_eq!(
            collect_max_fitnesses(&exp),
            vec![Some(200), Some(300), Some(400), Some(800)]
        );
        assert_eq!(collect_ranks(&exp), vec![2, 1, 0, 1]);
    }
    #[test]
    fn test_genome_initialization() {
        let num_genomes = 5;
        let mut settings = experiment_settings()
            .num_genomes(num_genomes)
            .build()
            .unwrap();
        let mut exp = SimpleExperiment::new(settings);

        let sample_genome = vec![0, 0, 0, 0, 0];

        exp.with_seed_genomes(vec![sample_genome.clone()]);
        exp.initialize();

        assert_eq!(exp.genome_entries.len(), num_genomes);
        assert_eq!(exp.genome_entries[0].genome, sample_genome.clone());
    }

    fn collect_ranks(exp: &SimpleExperiment) -> Vec<usize> {
        exp.genome_entries
            .iter()
            .map(|x| x.current_rank_score)
            .collect::<Vec<_>>()
    }
    fn collect_max_fitnesses(exp: &SimpleExperiment) -> Vec<Option<FitnessScore>> {
        exp.genome_entries
            .iter()
            .map(|x| x.max_fitness_metric)
            .collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct TrialResultItem {
    sim_unit_entry_id: UnitEntryId,

    // Refers to the index of the genome in the current experiment genome listing.  As such,
    // TrialResultItems are only valid data until the genome listing is updated.
    genome_idx: UnitEntryId,
    experiment_genome_uid: ExperimentGenomeUid,
    fitness_score: FitnessScore,
}

impl Debug for TrialResultItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "TrialResult(uid: {}, score: {})",
            self.experiment_genome_uid, self.fitness_score
        )
    }
}
