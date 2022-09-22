use std::{cell::Cell, rc::Rc, sync::mpsc, thread, time::Duration};

use rand::Rng;
use serde::Serialize;
use threadpool::ThreadPool;

use crate::{
    biology::{
        experiments::{
            alterations::{CompiledAlterationSet, GenomeAlterationImplementation},
            fitness::{calculate_new_fitness_ranks, normalize_ranks, ExperimentFitnessRank},
            sim_runner::{execute_sim_runners, ExperimentSimRunner, SimRunnerGenomeEntry},
            types::{
                CullStrategy, ExperimentGenomeUid, GenomeEntryId, GenomeExperimentEntry,
                TrialResultItem,
            },
            util::{
                cull_genomes, find_by_uid, highest_fitness_idx,
                partition_genomes_into_exaustive_groups, partition_genomes_into_subset_groups,
                partition_into_groups, partition_into_thirds, push_into_with_max,
                random_genome_of_length, scramble_groups, GenomeEntryInfo,
            },
        },
        genome::framed::{
            annotated::FramedGenomeExecutionStats,
            builders::FramedGenomeCompiler,
            common::{CompiledFramedGenome, FramedGenomeWord, RawFramedGenome},
        },
    },
    simulation::{
        common::{
            builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod, GeneticManifest,
        },
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

use super::types::{FitnessCycleStrategy, GenePoolSettings};

pub type GenePoolId = usize;

#[derive(Clone)]
pub struct ExperimentGenePool {
    pub id: GenePoolId,
    pub settings: GenePoolSettings,
    pub gm: Rc<GeneticManifest>,
    pub state: ExperimentGenePoolState,
}

#[derive(Clone)]
pub struct ExperimentGenePoolState {
    pub genome_entries: Vec<GenomeExperimentEntry>,
    pub eval_points: u64,
    _last_entry_id: usize,
    pub current_tick: u64,
    pub external_genomes_queue: Vec<CompiledFramedGenome>,
}

impl ExperimentGenePool {
    pub fn new(id: GenePoolId, settings: GenePoolSettings) -> Self {
        let gm = GeneticManifest::from_chemistry(&settings.sim_settings.chemistry_options.build());

        let mut s = Self {
            id,
            settings,
            state: ExperimentGenePoolState {
                genome_entries: vec![],
                _last_entry_id: 0,
                eval_points: 0,
                current_tick: 0,
                external_genomes_queue: vec![],
            },
            gm: Rc::new(gm),
        };

        s.initialize();

        s
    }

    pub fn initialize(&mut self) {
        self.populate_initial_genomes();
    }

    pub fn populate_initial_genomes(&mut self) {
        let mut rng = rand::thread_rng();

        match self.settings.seed_genome_settings {
            crate::biology::experiments::types::SeedGenomeSettings::Random {
                min_size,
                max_size,
            } => {
                while self.state.genome_entries.len() < self.settings.num_genomes {
                    self.register_new_genome(&random_genome_of_length(rng.gen_range((30..50))));
                }
            }
        };
    }

    pub fn add_eval_points(&mut self, eval_points: u64) {
        self.state.eval_points += eval_points;
    }

    pub fn execute_with_points(&mut self, eval_points: u64) {
        self.add_eval_points(eval_points);
        while self.state.eval_points > 0 {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        if self.state.current_tick != 0 {
            self.cull_and_replace();
        }

        perf_timer_start!("gp_groups");
        let groups = self.partition_into_groups();
        let groups = scramble_groups(groups, &self.settings.fitness_cycle_strategy);
        perf_timer_stop!("gp_groups");

        perf_timer_start!("gp_run_eval");
        let group_results = self.run_eval_for_groups(groups, false);
        perf_timer_stop!("gp_run_eval");

        for fitness_result in group_results.iter() {
            self.update_genomes_with_fitness_result(&fitness_result);
        }

        if self.state.current_tick % 100 == 0 {
            println!("gene pool {} tick: {}", self.id, self.state.current_tick);
        }

        self.state.current_tick += 1;

        // thread::sleep(Duration::from_millis(2000));
    }

    pub fn run_eval_for_groups(
        &mut self,
        groups: Vec<Vec<GenomeEntryInfo>>,
        use_threads: bool,
    ) -> Vec<Vec<TrialResultItem>> {
        perf_timer_start!("_make_runnable_genome_groups");
        let groups = self._make_runnable_genome_groups(groups);
        perf_timer_stop!("_make_runnable_genome_groups");

        // println!("running groups: {:?}", &groups);

        execute_sim_runners(
            groups,
            use_threads,
            &self.settings.sim_settings,
            &self.settings.fitness_calculation_key,
        )
    }

    pub fn _make_runnable_genome_groups(
        &mut self,
        groups: Vec<Vec<GenomeEntryInfo>>,
    ) -> Vec<Vec<SimRunnerGenomeEntry>> {
        groups
            .iter()
            .enumerate()
            .map_while(|(i, group)| {
                if self.state.eval_points > 0 {
                    let result = Some(
                        groups[i]
                            .iter()
                            .map(|genome_item| {
                                let entry = &self.state.genome_entries[genome_item.id];
                                assert_eq!(entry.uid, genome_item.uid);

                                SimRunnerGenomeEntry {
                                    gene_pool_id: self.id,
                                    genome_idx: genome_item.id,
                                    genome_uid: entry.uid,
                                    genome: entry.compiled_genome.as_ref().clone(),
                                    execution_stats: entry.previous_execution_stats.clone(),
                                }
                            })
                            .collect::<Vec<_>>(),
                    );

                    if self.state.eval_points >= group.len() as u64 {
                        self.state.eval_points -= group.len() as u64;
                    } else {
                        self.state.eval_points = 0;
                    }
                    result
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn partition_into_groups(&self) -> Vec<Vec<GenomeEntryInfo>> {
        let entry_items = self
            .state
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
            &self.settings.fitness_cycle_strategy,
            self.settings.sim_settings.num_genomes_per_sim,
        )
    }

    pub fn update_genomes_with_fitness_result(&mut self, fitness_results: &Vec<TrialResultItem>) {
        for trial_result in fitness_results.iter() {
            let genome_idx = self
                ._find_by_uid(trial_result.experiment_genome_uid)
                .unwrap();

            assert_eq!(genome_idx, trial_result.genome_idx);

            let mut max_fitness = self.state.genome_entries[genome_idx].max_fitness_metric;

            let mut is_new_max = false;
            if max_fitness.is_none() || trial_result.fitness_score > max_fitness.unwrap() {
                max_fitness = Some(trial_result.fitness_score);
                is_new_max = true;
            }

            self.state.genome_entries[genome_idx].max_fitness_metric = max_fitness;
            self.state.genome_entries[genome_idx].num_evaluations += 1;

            push_into_with_max(
                &mut self.state.genome_entries[genome_idx].last_fitness_metrics,
                trial_result.fitness_score,
                10,
            );

            self.state.genome_entries[genome_idx].previous_execution_stats =
                trial_result.stats.clone();

            // println!(
            //     "genome {} fitness: {}",
            //     trial_result.genome_idx, trial_result.fitness_score
            // );
        }

        let new_ranks = calculate_new_fitness_ranks(
            &fitness_results
                .iter()
                .map(|res| {
                    (
                        res.clone(),
                        self.state.genome_entries[res.genome_idx].current_rank_score,
                    )
                })
                .collect::<Vec<_>>(),
            &self.settings.fitness_rank_adjustment_method,
        );

        for new_rank in new_ranks {
            // println!(
            //     "fitness: {}, rank: {}, id: {}",
            //     new_rank.0.fitness_score, new_rank.1, new_rank.0.genome_idx
            // );

            assert_eq!(
                self._find_by_uid(new_rank.0.experiment_genome_uid).unwrap(),
                new_rank.0.genome_idx
            );

            self.state.genome_entries[new_rank.0.genome_idx].current_rank_score = new_rank.1;
        }

        self.normalize_ranks();
    }

    pub fn normalize_ranks(&mut self) {
        let mut ranks = self
            .state
            .genome_entries
            .iter()
            .enumerate()
            .map(|(i, genome)| (i, genome.current_rank_score))
            .collect::<Vec<_>>();

        normalize_ranks(&mut ranks);

        for i in 0..ranks.len() {
            self.state.genome_entries[ranks[i].0].current_rank_score = ranks[i].1;
        }
    }

    pub fn cull_and_replace(&mut self) {
        cull_genomes(
            &mut self.state.genome_entries,
            &self.settings.cull_strategy,
            self.settings.num_genomes,
        );

        while self.state.external_genomes_queue.len() > 0
            && self.state.genome_entries.len() < self.settings.num_genomes
        {
            let genome = self.state.external_genomes_queue.remove(0);
            self.register_new_genome(&genome.raw_values);
        }

        let raw_genomes = pull_fresh_genomes(
            &mut self.state.genome_entries,
            self.settings.num_genomes,
            &self.settings.alteration_specs,
        );

        for raw_genome in &raw_genomes {
            self.register_new_genome(raw_genome);
        }
    }

    pub fn register_new_genome(&mut self, genome: &RawFramedGenome) {
        let next_genome_id = if self.state.genome_entries.len() > 0 {
            self.state._last_entry_id + 1
        } else {
            0
        };

        let compiled_genome = FramedGenomeCompiler::compile(genome.clone(), &self.gm).wrap_rc();
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

        self.state._last_entry_id = genome_entry.uid;

        if self
            .state
            .genome_entries
            .iter()
            .any(|e| e.uid == genome_entry.uid)
        {
            panic!("uid is duplicated");
        }
        self.state.genome_entries.push(genome_entry);
    }

    fn _find_by_uid(&self, uid: ExperimentGenomeUid) -> Option<usize> {
        find_by_uid(&self.state.genome_entries, uid)
    }

    pub fn _highest_fitness_idx(&self) -> usize {
        highest_fitness_idx(&self.state.genome_entries)
    }
}

pub fn pull_fresh_genomes(
    genomes: &mut Vec<GenomeExperimentEntry>,
    target_count: usize,
    alteration_set: &CompiledAlterationSet,
) -> Vec<Vec<FramedGenomeWord>> {
    let mut sorted_by_rank = genomes
        .iter()
        .map(|genome| (genome.uid, genome.current_rank_score))
        .collect::<Vec<_>>();
    sorted_by_rank.sort_by_cached_key(|g| g.1);

    let mut raw_genomes = vec![];
    while genomes.len() + raw_genomes.len() < target_count {
        let alteration = choose_random_alteration(alteration_set);
        let genome = pull_fresh_genome(genomes, &alteration, &sorted_by_rank);
        if genome.len() > 0 {
            raw_genomes.push(genome);
        }
    }

    raw_genomes
}

pub fn choose_random_alteration(
    alterations_set: &CompiledAlterationSet,
) -> GenomeAlterationImplementation {
    let mut rng = rand::thread_rng();
    let alt_i = rng.gen_range((0..alterations_set.alterations.len()));
    alterations_set.alterations[alt_i].clone()
}

fn pull_fresh_genome(
    genomes: &Vec<GenomeExperimentEntry>,
    alteration: &GenomeAlterationImplementation,
    sorted_by_fitness: &Vec<(ExperimentGenomeUid, ExperimentFitnessRank)>,
) -> Vec<FramedGenomeWord> {
    let mut input_genomes = vec![];
    for i in (0..alteration.genomes_required) {
        let uid = select_random_top_genome(sorted_by_fitness);

        let (idx, g) = genomes
            .iter()
            .enumerate()
            .find(|(idx, g)| g.uid == uid)
            .unwrap();

        // println!(
        //     "continuing genome {} with max_fitness {:?}",
        //     genomes[idx].uid, genomes[idx].max_fitness_metric
        // );

        input_genomes.push(genomes[idx].compiled_genome.as_ref());
    }

    let params = (alteration.prepare)(&input_genomes.as_slice());
    let new_genome = (alteration.execute)(&input_genomes.as_slice(), &params.as_slice());

    new_genome
}

pub fn select_random_top_genome(
    sorted_genomes: &Vec<(ExperimentGenomeUid, ExperimentFitnessRank)>,
) -> ExperimentGenomeUid {
    let mut rng = rand::thread_rng();

    let start = sorted_genomes.len() / 2;
    let i = rng.gen_range((start..sorted_genomes.len()));

    sorted_genomes[i].0
}
