pub mod logger;
pub mod utils;

use crate::biology::experiments::alterations;
use crate::biology::experiments::fitness::normalize_ranks;
use crate::biology::experiments::sim_runner::{execute_sim_runners, SimRunnerGenomeEntry};
use crate::biology::experiments::types::{
    CullStrategy, ExperimentGenomeUid, GenomeExperimentEntry, TrialResultItem,
};
use crate::biology::experiments::util::{
    cull_genomes, get_mean_fitness_for_genome, highest_fitness_idx, partition_into_groups,
    push_into_with_max, random_genome_of_length, scramble_groups, GenomeEntryInfo,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::annotated::FramedGenomeExecutionStats;
use crate::biology::genome::framed::common::*;
use crate::simulation::fitness::FitnessScore;
use crate::util::RateCounter;
use rand::Rng;

use self::logger::SimpleExperimentLogger;
use self::utils::SimpleExperimentSettings;

use super::multi_pool::gene_pool::pull_fresh_genomes;
use super::multi_pool::types::FitnessCycleStrategy;

macro_rules! is_experiment_logging_enabled {
    ( ) => {
        // true
        false
    };
}

macro_rules! explog {
    ($($arg:tt)*) => ({
        if is_experiment_logging_enabled!() {println!($($arg)*)} else {}
    })
}

pub struct SimpleExperiment {
    pub is_paused: bool,
    pub is_initialized: bool,
    pub genome_entries: Vec<GenomeExperimentEntry>,
    pub current_tick: u64,

    pub settings: Rc<SimpleExperimentSettings>,
    pub _last_entry_id: usize,

    _gm: Rc<GeneticManifest>, // a cached copy.  note that this might eventually change depending on the genome.
    _logger: Option<SimpleExperimentLogger>,
    _seed_genomes: Option<Vec<RawFramedGenome>>,

    _rate_counter: RateCounter,
}

impl SimpleExperiment {
    pub fn new(settings: SimpleExperimentSettings) -> Self {
        let logger = settings
            .logging_settings
            .as_ref()
            .map(|settings| SimpleExperimentLogger {
                settings: settings.clone(),
            });

        let chemistry = settings.sim_settings.chemistry_options.build();
        let gm = GeneticManifest::from_chemistry(&chemistry);
        SimpleExperiment {
            current_tick: 0,
            is_paused: true,
            is_initialized: false,
            genome_entries: vec![],
            settings: Rc::new(settings),
            _last_entry_id: 0,

            _gm: Rc::new(gm),
            _logger: logger,
            _seed_genomes: None,
            _rate_counter: RateCounter::new(),
        }
    }

    pub fn initialize(&mut self) {
        if let Some(logger) = &self._logger {
            logger.init();
            logger.log_settings(&self.settings);
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
            logger.log_status(
                self.current_tick,
                &self.genome_entries,
                &self._gm,
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

    pub fn tick(&mut self) {
        perf_timer_start!("experiment_partition");
        let groups = self.partition_into_groups();
        let groups = scramble_groups(groups, &self.settings.fitness_cycle_strategy);

        explog!("groups: {:?}", &groups);
        perf_timer_stop!("experiment_partition");

        perf_timer_start!("run_eval_for_groups");
        let group_results = self.run_eval_for_groups(groups, true);
        perf_timer_stop!("run_eval_for_groups");

        perf_timer_start!("update_fitness_result");
        for fitness_result in group_results.iter() {
            self.update_genomes_with_fitness_result(&fitness_result);
        }
        perf_timer_stop!("update_fitness_result");
        // perf_timer_start!("adjust_ranks");
        // perf_timer_stop!("adjust_ranks");

        if self.current_tick > 0 && self.current_tick % 2000 == 0 {
            println!("num genomes: {}", self.genome_entries.len());
            self.print_fitness_summary();
        }

        explog!("status after adjusting: {:?}", &self.__gather_status());
        perf_timer_start!("experiment_cull_and_replace");
        self.cull_and_replace();
        perf_timer_stop!("experiment_cull_and_replace");
        explog!("status after culling: {:?}", &self.__gather_status());

        self.post_tick_logging();
    }

    pub fn post_tick_logging(&mut self) {
        if let Some(logger) = &self._logger {
            perf_timer_start!("experiment_logging");

            logger.log_fitness_percentiles(
                self.current_tick as u64,
                &self.genome_entries,
                self.settings.iterations,
            );

            // let checkpoint_interval = logger.settings.checkpoint_interval * factor;
            let _tick = self.current_tick + 1;

            if self.current_tick % 2000 == 0 {
                logger.log_status(_tick as u64, &self.genome_entries, &self._gm);
                logger.log_best_genomes(
                    _tick as u64,
                    &self.genome_entries,
                    self.settings.sim_settings.num_genomes_per_sim,
                );
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

        self._rate_counter.increment();

        if self.current_tick % 1000 == 0 && self.current_tick != 0 {
            self._rate_counter.calculate_and_print_rate();
        }
    }

    pub fn partition_into_groups(&mut self) -> Vec<Vec<GenomeEntryInfo>> {
        let entry_items = self
            .genome_entries
            .iter()
            .enumerate()
            .map(|(i, genome)| GenomeEntryInfo {
                id: i,
                uid: genome.uid,
                fitness_rank: genome.current_rank_score,
            })
            .collect::<Vec<_>>();

        partition_into_groups(
            entry_items,
            &super::multi_pool::types::FitnessCycleStrategy::Exaustive {
                group_scramble_pct: 0.30,
            },
            self.settings.sim_settings.num_genomes_per_sim,
        )
    }

    pub fn normalize_ranks(&mut self) {
        let mut ranks = self
            .genome_entries
            .iter()
            .enumerate()
            .map(|(i, genome)| (i, genome.current_rank_score))
            .collect::<Vec<_>>();

        normalize_ranks(&mut ranks);

        for i in 0..ranks.len() {
            self.genome_entries[ranks[i].0].current_rank_score = ranks[i].1;
        }

        // // let highest_rank = self.genome_entries.len() - 1;

        // // let mut entries = self.genome_entries.clone();
        // // entries.sort_by_key(|entry| entry.current_rank_score);

        // //normalize
        // let mut sorted_ranks = self
        //     .genome_entries
        //     .iter()
        //     .enumerate()
        //     .map(|(i, entry)| (entry.uid, entry.current_rank_score))
        //     .collect::<Vec<_>>();
        // sorted_ranks.sort_by_cached_key(|(genome_id, score)| *score);

        // // explog!("normalizing: {:?}", &sorted_ranks);

        // let mut current_rank_tally = 0;
        // let mut prev_rank = None;
        // for i in (0..sorted_ranks.len()) {
        //     let _current = sorted_ranks[i].1;
        //     if prev_rank != None && prev_rank != Some(_current) {
        //         current_rank_tally += 1;
        //     }
        //     // explog!(
        //     //     "[i:{}] rank_tally: {:?} -- current_rank: {} -- prev_rank: {:?}",
        //     //     i,
        //     //     current_rank_tally,
        //     //     _current,
        //     //     prev_rank
        //     // );
        //     sorted_ranks[i].1 = current_rank_tally;

        //     prev_rank = Some(_current);
        // }

        // for i in (0..sorted_ranks.len()) {
        //     let uid = sorted_ranks[i].0;
        //     let rank = sorted_ranks[i].1;
        //     let index = self._find_by_uid(uid).unwrap();
        //     self.genome_entries[index].current_rank_score = rank;
        // }
    }

    pub fn update_genomes_with_fitness_result(&mut self, fitness_result: &Vec<TrialResultItem>) {
        let mut sorted_result = fitness_result.clone();
        sorted_result.sort_by_cached_key(|x| x.fitness_score);

        // println!("processing fitness result: {:?}", fitness_result);
        // self.print_fitness_summary();
        for (result_rank, fitness_result_item) in sorted_result.iter().enumerate() {
            let our_genome_uid = fitness_result_item.experiment_genome_uid;
            let our_genome_idx = self._find_by_uid(our_genome_uid).unwrap();
            assert_eq!(our_genome_idx, fitness_result_item.genome_idx);

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
            self.genome_entries[our_genome_idx].num_evaluations += 1;

            push_into_with_max(
                &mut self.genome_entries[our_genome_idx].last_fitness_metrics,
                our_fitness_score,
                10,
            );

            self.genome_entries[our_genome_idx].previous_execution_stats =
                fitness_result_item.stats.clone();
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
        cull_genomes(
            &mut self.genome_entries,
            &CullStrategy::WorstFirst { percent: 0.30 },
            self.settings.num_genomes,
        );

        let raw_genomes = pull_fresh_genomes(
            &mut self.genome_entries,
            self.settings.num_genomes,
            &self.settings.alteration_set,
        );

        for raw_genome in raw_genomes {
            self.register_new_genome(raw_genome);
        }
    }

    // pub fn pull_fresh_genome(&mut self) {
    //     let mut rng = rand::thread_rng();
    //     let alt_i = rng.gen_range((0..self.settings.alteration_set.alterations.len()));
    //     self._pull_fresh_genome(alt_i);
    // }

    // // TODO: something about this is slow
    // fn _pull_fresh_genome(&mut self, alteration_index: usize) {
    //     //explog!("Pulling new genome...");
    //     let alteration = &self.settings.alteration_set.alterations[alteration_index].clone();

    //     let mut genomes = vec![];
    //     for i in (0..alteration.genomes_required) {
    //         let uid = self._select_random_top_genome();
    //         let index = self._find_by_uid(uid).unwrap();

    //         genomes.push(self.genome_entries[index].compiled_genome.as_ref());
    //     }

    //     let params = (alteration.prepare)(&genomes.as_slice());
    //     let new_genome = (alteration.execute)(&genomes.as_slice(), &params.as_slice());

    //     if new_genome.len() > 0 {
    //         self.register_new_genome(new_genome);
    //     }
    // }

    pub fn register_new_genome(&mut self, genome: RawFramedGenome) {
        let next_genome_id = if self.genome_entries.len() > 0 {
            self._last_entry_id + 1
        } else {
            0
        };

        let compiled_genome = FramedGenomeCompiler::compile(genome.clone(), &self._gm).wrap_rc();
        let stats = FramedGenomeExecutionStats::new(&compiled_genome.frames);

        let genome_entry = GenomeExperimentEntry {
            last_fitness_metrics: vec![],
            max_fitness_metric: None,
            num_evaluations: 0,
            compiled_genome: compiled_genome,
            uid: next_genome_id as ExperimentGenomeUid,
            current_rank_score: 0,
            previous_execution_stats: stats,
        };

        self._last_entry_id = genome_entry.uid;

        if self
            .genome_entries
            .iter()
            .any(|e| e.uid == genome_entry.uid)
        {
            panic!("uid is duplicated");
        }
        self.genome_entries.push(genome_entry);
    }

    // pub fn _find_newest_genome_uid(&self) -> ExperimentGenomeUid {
    //     let mut min_i = 0;
    //     let mut min_evals = None;
    //     for (i, genome) in self.genome_entries.iter().enumerate() {
    //         if min_evals.is_none() || genome.num_evaluations < min_evals.clone().unwrap() {
    //             min_evals = Some(genome.num_evaluations);
    //             min_i = i;
    //         }
    //     }

    //     self.genome_entries[min_i].uid
    // }

    // fn _select_random_top_genome(&self) -> usize {
    //     let mut rng = rand::thread_rng();
    //     let i = rng.gen_range((0..self.genome_entries.len()));
    //     let mut by_rank = self.genome_entries.clone();
    //     by_rank.sort_by_cached_key(|entry| entry.current_rank_score);
    //     by_rank[i].uid
    // }
    fn _find_by_uid(&self, uid: ExperimentGenomeUid) -> Option<usize> {
        #[cfg(debug_assertions)]
        {
            let count = self.genome_entries.iter().filter(|e| e.uid == uid).count();
            if count > 1 {
                panic!("duplicate");
            }
        }

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

        statuses.sort_by_cached_key(|x| x[2]);
        statuses.reverse();
        statuses
    }

    // // TODO: use the generalized form of this
    // pub fn scramble_groups(
    //     &self,
    //     mut groups: Vec<Vec<(GenomeEntryId, ExperimentGenomeUid)>>,
    // ) -> Vec<Vec<(GenomeEntryId, ExperimentGenomeUid)>> {
    //     if groups.len() == 0 || groups[0].len() == 0 {
    //         return groups;
    //     }

    //     use rand::Rng;
    //     let mut rng = rand::thread_rng();

    //     for i in 0..groups.len() {
    //         let to_shuffle: usize = (groups[i].len() as f64 * 0.30).round() as usize;

    //         for count in 0..to_shuffle {
    //             let dest_group = rng.gen_range(0..groups.len());

    //             let src_group_length = groups[i].len();
    //             let removed = groups[i].remove(rng.gen_range(0..src_group_length));
    //             groups[dest_group].push(removed);

    //             let dest_group_length = groups[dest_group].len();
    //             let removed = groups[dest_group].remove(rng.gen_range(0..dest_group_length));
    //             groups[i].push(removed);
    //         }
    //     }

    //     // while redeal.len() != 0 {
    //     //     let i = rng.gen_range(0..groups.len());
    //     //     groups[i].push(redeal.remove(0));
    //     // }
    //     // println!(
    //     //     "GROUPS: {:?}",
    //     //     groups // groups.iter().map(|g| g.len()).collect::<Vec<_>>()
    //     // );
    //     groups
    // }

    pub fn run_eval_for_groups(
        &self,
        groups: Vec<Vec<GenomeEntryInfo>>,
        use_threads: bool,
    ) -> Vec<Vec<TrialResultItem>> {
        let groups = groups
            .iter()
            .enumerate()
            .map(|(i, group)| {
                groups[i]
                    .iter()
                    .map(|(entry_info)| {
                        assert_eq!(self.genome_entries[entry_info.id].uid, entry_info.uid);

                        SimRunnerGenomeEntry {
                            gene_pool_id: 0, // not used
                            genome_idx: entry_info.id,
                            genome_uid: entry_info.uid,
                            genome: self.genome_entries[entry_info.id]
                                .compiled_genome
                                .as_ref()
                                .clone(),
                            execution_stats: self.genome_entries[entry_info.id]
                                .previous_execution_stats
                                .clone(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        execute_sim_runners(
            groups,
            use_threads,
            &self.settings.sim_settings,
            &self.settings.fitness_calculation_key,
        )
    }

    fn get_max_fitness_for_genome(&self, uid: ExperimentGenomeUid) -> Option<FitnessScore> {
        let idx = self._find_by_uid(uid).unwrap();
        self.genome_entries[idx].max_fitness_metric
    }

    fn print_fitness_summary(&self) {
        let mut entries = self.genome_entries.clone();
        entries.sort_by_cached_key(|entry| entry.current_rank_score);
        let summary = entries.iter().enumerate().map(|(idx, entry)| {
            (
                entry.uid,
                entry.current_rank_score,
                entry.max_fitness_metric,
                entry.last_fitness_metrics.clone(),
            )
        });

        for line in summary {
            println!(
                "[uid: {}] <{}> max: {:?}, recent:{:?}",
                &line.0, &line.1, &line.2, line.3,
            );
        }
        println!("");
    }

    fn _highest_fitness_idx(&self) -> usize {
        highest_fitness_idx(&self.genome_entries)
    }

    // fn inject_special_genome(&mut self) {
    //     self._last_entry_id += 1;
    //     let genome = get_genome2_raw(&self._gm);

    //     // println!(
    //     //     "INJECTING: {}",
    //     //     FramedGenomeCompiler::compile(genome.clone(), &self._gm).display(&self._gm)
    //     // );
    //     let genome = GenomeExperimentEntry {
    //         last_fitness_metrics: vec![],
    //         max_fitness_metric: None,
    //         num_evaluations: 0,
    //         compiled_genome: FramedGenomeCompiler::compile(genome.clone(), &self._gm).wrap_rc(),
    //         raw_genome: genome,
    //         uid: usize::MAX,
    //         current_rank_score: 10,
    //     };

    //     self.genome_entries[0] = genome.clone();
    //     self.genome_entries[1] = genome.clone();
    // }
}

#[cfg(test)]
pub mod tests {
    use variants::CheeseChemistry;

    use self::utils::SimpleExperimentSettingsBuilder;
    use super::*;
    use crate::biology::experiments::types::{CullStrategy, ExperimentSimSettings};
    use crate::biology::experiments::util::random_genome_of_length;
    use crate::biology::genetic_manifest::GeneticManifest;
    use crate::simulation::common::builder::ChemistryBuilder;
    use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
    use crate::simulation::common::*;

    pub fn experiment_settings() -> SimpleExperimentSettingsBuilder {
        let chemistry_builder = ChemistryBuilder::with_key("cheese");
        let chemistry = chemistry_builder.build();

        let gm =
            GeneticManifest::construct::<CheeseChemistry>(&chemistry.get_configuration()).wrap_rc();

        SimpleExperimentSettingsBuilder::default()
            .alteration_set(alterations::default_alteration_set())
            .experiment_key("my_experiment".to_string())
            .cull_strategy(CullStrategy::WorstFirst { percent: 0.30 })
            .fitness_calculation_key("total_cheese_acquired".to_string())
            .num_genomes(11)
            .sim_settings(ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                default_unit_resources: vec![("cheese".to_owned(), 200)],
                default_unit_attr: vec![],
                place_units_method: PlaceUnitsMethod::Default,
                chemistry_options: chemistry_builder,
            })
            .fitness_cycle_strategy(FitnessCycleStrategy::Exaustive {
                group_scramble_pct: 0.30,
            })
            .iterations(1)
            .logging_settings(None)
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

        let groups = groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|info| (info.id, info.uid))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        println!("groups: {:?}", groups);

        let flattened = groups.iter().flatten().map(|x| *x).collect::<Vec<_>>();
        assert_eq!(flattened.len(), num_genomes + 1); //because the genome count isn't a multiple of the group size, there's double inclusion

        assert_eq!(
            flattened,
            vec![(4, 4), (3, 3), (2, 2), (1, 1), (1, 1), (0, 0)]
        );
        assert_eq!(
            groups,
            vec![
                vec![(4, 4), (3, 3)],
                vec![(2, 2), (1, 1)],
                vec![(1, 1), (0, 0)]
            ]
        );
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
                gene_pool_id: 0,
                genome_idx: 0,
                sim_unit_entry_id: 0,
                experiment_genome_uid: 0,
                fitness_score: 200,
                stats: FramedGenomeExecutionStats::empty(),
            },
            TrialResultItem {
                gene_pool_id: 0,
                genome_idx: 1,
                sim_unit_entry_id: 1,
                experiment_genome_uid: 1,
                fitness_score: 125,
                stats: FramedGenomeExecutionStats::empty(),
            },
            TrialResultItem {
                gene_pool_id: 0,
                genome_idx: 2,
                sim_unit_entry_id: 2,
                experiment_genome_uid: 2,
                fitness_score: 100,
                stats: FramedGenomeExecutionStats::empty(),
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
        assert_eq!(
            exp.genome_entries[0].compiled_genome.raw_values,
            sample_genome.clone()
        );
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
