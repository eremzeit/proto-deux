use crate::biology::phenotype::framed::common::*;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use std::fmt::{Debug, Formatter, Result};

pub mod alterations;
pub mod fitness;

pub type _ExperimentGenomeId = usize;
pub type ExperimentGenomeUid = usize;

macro_rules! is_experiment_logging_enabled {
    //( ) => { false }
    ( ) => {
        true
    };
}

#[macro_export]
macro_rules! explog {
    ($($arg:tt)*) => ({
        if is_experiment_logging_enabled!() {println!($($arg)*)} else {}
    })
}

#[derive(Clone)]
pub struct GenomeExperimentEntry {
    pub last_fitness_metrics: Vec<FitnessScore>,
    pub max_fitness_metric: Option<FitnessScore>,
    pub num_evaluations: usize,
    pub genome: Vec<FramedGenomeWord>,
    pub uid: ExperimentGenomeUid,
    pub current_rank_score: usize,
}

pub struct ExperimentSimSettings {
    pub num_simulation_ticks: u64,
    pub grid_size: (usize, usize),
    pub num_genomes_per_sim: usize,
    pub iterations: usize,
    pub default_unit_resources: Vec<(&'static str, UnitResourceAmount)>,
    pub default_unit_attr: Vec<(&'static str, UnitAttributeValue)>,
}

//default resources =>>> vec![("cheese", 100)]

// PlaceUnitsMethod::SimpleDrop {
//     attributes: None,
// }
pub struct SimpleExperimentSettings {
    pub num_genomes: usize,
    pub iterations: usize,
    pub sim_settings: ExperimentSimSettings,
    pub genetic_manifest: GeneticManifest,
    pub sensor_manifest: SensorManifest,
    pub chemistry_key: String,
    pub alteration_set: alterations::AlterationTypeSet,
    pub fitness_calculation_key: String,
    pub cull_strategy: CullStrategy,
}

pub struct SimpleExperiment {
    pub ticks: u64,
    pub is_paused: bool,
    pub genome_entries: Vec<GenomeExperimentEntry>,
    pub is_headless: bool,
    pub current_tick: u64,
    pub settings: SimpleExperimentSettings,
    pub _last_entry_id: usize,
}

pub type GenomeEntryId = usize;

pub enum CullStrategy {
    WorstFirst,
}

impl SimpleExperiment {
    pub fn new(settings: SimpleExperimentSettings) -> Self {
        SimpleExperiment {
            ticks: 0,
            current_tick: 0,
            is_paused: true,
            genome_entries: vec![],
            is_headless: true,
            settings,
            _last_entry_id: 0,
        }
    }

    pub fn resume(&mut self) {
        while self.current_tick < self.settings.iterations as u64 {
            self.tick();
            self.current_tick += 1;
        }
        explog!("FINAL STATUS: {:?}", &self.__gather_status());
    }

    pub fn _find_newest_genome_uid(&self) -> usize {
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

    pub fn partition_into_groups(&mut self) -> Vec<Vec<usize>> {
        println!(
            "partitioning uids: {:?}",
            self.genome_entries
                .iter()
                .map(|unit| { unit.uid })
                .collect::<Vec<_>>()
        );
        let num_genomes = self.genome_entries.len();
        let group_size = self.settings.sim_settings.num_genomes_per_sim;

        let mut entries_by_fitness = self.genome_entries.clone();
        entries_by_fitness.sort_by_key(|entry| entry.current_rank_score);

        let mut partitions = vec![];
        let mut num_groups = num_genomes / group_size;
        if num_genomes % group_size > 0 {
            num_groups += 1;
        }

        println!(
            "uids in entries_by_fitness: {:?}",
            entries_by_fitness
                .iter()
                .map(|unit| { unit.uid })
                .collect::<Vec<_>>()
        );
        let mut genome_idx = 0;
        for group_idx in (0..num_groups) {
            let mut group = vec![];

            for i in (0..group_size) {
                if genome_idx < num_genomes {
                    group.push(entries_by_fitness[genome_idx].uid);
                    genome_idx += 1;
                } else {
                    group.push(self._find_newest_genome_uid())
                }
            }

            partitions.push(group);
        }
        //let flattened = partitions.iter().flatten().collect::<Vec<_>>();
        //println!("flattened groups: {:?}", flattened.len());
        partitions
    }

    pub fn seed_random_genomes(&mut self, num: usize) {
        let mut genomes = vec![];
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for i in (0..num) {
            genomes.push(random_genome_of_length(rng.gen_range((30..50))));
        }

        let entries = genomes
            .iter()
            .enumerate()
            .map(|(i, genome)| GenomeExperimentEntry {
                last_fitness_metrics: vec![],
                max_fitness_metric: None,
                num_evaluations: 0,
                genome: genome.clone(),
                uid: (self._last_entry_id + i) as ExperimentGenomeUid,
                current_rank_score: 0,
            })
            .collect::<Vec<_>>();
        self._last_entry_id += self.genome_entries.len();
        self.genome_entries = entries;
    }

    pub fn _summarize_genomes(
        &self,
        cm: &ChemistryManifest,
        gm: &GeneticManifest,
        sm: &SensorManifest,
    ) -> String {
        let mut s = "".to_string();

        self.genome_entries.iter().map(|entry| {
            let genome_vals = entry.genome.clone();
            let genome = FramedGenomeParser::parse(genome_vals, cm.clone(), sm.clone(), gm.clone());
            s = format!("{}\n{}", s, genome.display(sm, cm, gm));
        });
        s
    }

    pub fn _get_unit_entries_for_uids(
        &self,
        uids: &[ExperimentGenomeUid],
        cm: &ChemistryManifest,
        sm: &SensorManifest,
        gm: &GeneticManifest,
    ) -> Vec<UnitEntry> {
        let mut unit_entries = vec![];

        let mut count = 0;
        for uid in uids {
            let maybe_idx = self._find_by_uid(*uid);
            let idx = maybe_idx.unwrap();
            let genome_vals = self.genome_entries[idx].genome.clone();
            let genome = FramedGenomeParser::parse(genome_vals, cm.clone(), sm.clone(), gm.clone());

            let unit_entry = UnitEntryBuilder::default()
                .species_name(format!("species: {}", count))
                .phenotype(
                    FramedGenomePhenotype::new(genome, gm.clone(), cm.clone(), sm.clone())
                        .construct(),
                )
                .default_resources(self.settings.sim_settings.default_unit_resources.clone())
                .default_attributes(self.settings.sim_settings.default_unit_attr.clone())
                .build(&cm, None);

            unit_entries.push(unit_entry);
            count += 1;
        }

        unit_entries
    }

    pub fn run_evaluation_for_uids(
        &mut self,
        genome_uids: &Vec<ExperimentGenomeUid>,
    ) -> Vec<TrialResultItem> {
        let chemistry = get_chemistry_by_key(
            &self.settings.chemistry_key.to_string(),
            PlaceUnitsMethod::Default,
        );
        let sm = SensorManifest::with_default_sensors(chemistry.get_manifest());
        let gm = &self.settings.genetic_manifest;
        let unit_entries = self._get_unit_entries_for_uids(
            genome_uids.as_slice(),
            chemistry.get_manifest(),
            &sm,
            gm,
        );
        //explog!("EVAL fitness for genomes: {:?}", genome_uids);

        let mut sim = SimulationBuilder::default()
            .headless(self.is_headless)
            .size(self.settings.sim_settings.grid_size.clone())
            .iterations(self.settings.sim_settings.num_simulation_ticks)
            .chemistry_key(self.settings.chemistry_key.clone())
            // .unit_placement()
            .unit_manifest(UnitManifest {
                units: unit_entries,
            })
            .to_simulation();

        let mut executor = SimpleSimulationExecutor::new(sim);
        executor.start();

        let mut fitness_scores = vec![];
        let unit_entries = executor.simulation.unit_manifest.units.clone();
        assert_eq!(unit_entries.len(), genome_uids.len());
        for i in (0..genome_uids.len()) {
            let entry = &unit_entries[i];
            let sim_unit_entry_id = entry.info.id;
            let genome_uid = genome_uids[i as usize];
            let genome_idx = self._find_by_uid(genome_uid).unwrap();

            self.genome_entries[genome_idx].num_evaluations += 1;

            // println!("fitness key: {}", self.settings.fitness_calculation_key);
            let fitness_score = calculate_fitness(
                &self.settings.fitness_calculation_key,
                entry.info.id,
                &mut executor.simulation.editable(),
            );
            // use rand::Rng;
            // let mut rng = rand::thread_rng();
            // let fitness_score = ((genome_uid % 10) + rng.gen_range(0..2)) as FitnessScore;

            let resultItem = TrialResultItem {
                sim_unit_entry_id,
                genome_uid,
                genome_idx,
                fitness_score,
            };
            fitness_scores.push(resultItem);
        }

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

    pub fn adjust_ranks_based_on_result(&mut self, fitness_result: &Vec<TrialResultItem>) {
        let mut sorted_result = fitness_result.clone();
        sorted_result.sort_by_key(|x| x.fitness_score);
        //sorted_result.reverse();
        // println!("sorted: {:?}", sorted_result);

        for (result_rank, fitness_result_item) in sorted_result.iter().enumerate() {
            let our_genome_idx = fitness_result_item.genome_idx;
            let our_genome_uid = fitness_result_item.genome_uid;
            let our_fitness_score = fitness_result_item.fitness_score;
            //let genome_idx = self._find_by_uid(*genome_uid).unwrap() as usize;
            let existing_max = self.genome_entries[our_genome_idx].max_fitness_metric;
            let mut new_max_fitness = existing_max;

            match existing_max {
                Some(_max_fitness) => {
                    let fitness_diff_from_max = _max_fitness as i64 - our_fitness_score as i64;
                    if fitness_diff_from_max < 0 {
                        new_max_fitness = Some(our_fitness_score);
                    }
                }
                None => {
                    new_max_fitness = Some(our_fitness_score);
                }
            }

            self.genome_entries[our_genome_idx].max_fitness_metric = new_max_fitness;

            // println!(
            //     "processing fitness for id: {} with score {}",
            //     genome_id, self.genome_entries[*genome_id as usize].current_rank_score
            // );

            let our_rank_score = self.genome_entries[our_genome_idx].current_rank_score;

            for i in (0..sorted_result.len()) {
                let their_uid = sorted_result[i].genome_uid;
                let their_genome_idx = sorted_result[i].genome_idx;

                let their_rank_score = self.genome_entries[their_genome_idx].current_rank_score;
                let their_fitness_score = sorted_result[i].fitness_score;

                let mut our_new_rank_score = self.genome_entries[our_genome_idx].current_rank_score;
                //println!("\t...comparing to {} with score {}", id, loser_rank_score);
                let is_upset =
                    our_fitness_score > their_fitness_score && our_rank_score <= their_rank_score;
                if is_upset {
                    our_new_rank_score = their_rank_score + 1;
                    // explog!(
                    //     "\t-> {} (r:{}, {}) upsets against {} (r:{}, {}) -- new rank: {}",
                    //     our_genome_uid,
                    //     our_rank_score,
                    //     our_fitness_score,
                    //     their_uid,
                    //     their_rank_score,
                    //     their_fitness_score,
                    //     our_new_rank_score
                    // );
                }
                self.genome_entries[our_genome_idx].current_rank_score = our_new_rank_score;
            }
        }
        self.normalize_ranks();
    }

    pub fn initialize(&mut self) {
        self.seed_random_genomes(self.settings.num_genomes);
    }
    pub fn cull_and_replace(&mut self) {
        let target_count = (self.settings.num_genomes as f64 * 0.90) as usize;
        let to_remove = self.genome_entries.len() - target_count;
        let mut by_rank = self.genome_entries.clone();
        by_rank.sort_by_key(|entry| entry.current_rank_score);

        let mut removed_count = 0;
        let mut uids_to_remove = vec![];

        let mut offset_from_bottom = 0;
        while removed_count < to_remove && offset_from_bottom < self.genome_entries.len() {
            while offset_from_bottom < self.genome_entries.len()
                && by_rank[offset_from_bottom].max_fitness_metric.is_none()
            {
                offset_from_bottom += 1;
            }
            let item = by_rank.remove(offset_from_bottom);
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
            self.genome_entries.push(GenomeExperimentEntry {
                last_fitness_metrics: vec![],
                max_fitness_metric: None,
                num_evaluations: 0,
                genome: new_genome,
                uid: self._last_entry_id + 1,
                current_rank_score: 0,
            });

            self._last_entry_id += 1;
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

    pub fn tick(&mut self) {
        explog!("EXPERIMENT TICK");
        let groups = self.partition_into_groups();
        explog!("groups: {:?}", &groups);

        for group in groups {
            let fitness_result = self.run_evaluation_for_uids(&group);
            // println!("fitness_scores: {:?}", fitness_result);
            self.adjust_ranks_based_on_result(&fitness_result);
        }

        //println!("status after adjusting: {:?}", &self.__gather_status());
        self.cull_and_replace();
        //println!("status after culling: {:?}", &self.__gather_status());

        let mut max_scores = self
            .genome_entries
            .iter()
            .map(|entry| (entry.uid, entry.max_fitness_metric))
            .collect::<Vec<_>>();
        max_scores.sort_by_key(|entry| entry.1);
        max_scores.reverse();

        // pick a genome in the top 50% and mutate it.  test the fitness
        // and if it is better than any existing genomes, replace one of those.
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
    use super::*;
    use crate::biology::genome::framed::common::*;
    use crate::biology::phenotype::framed::common::*;
    use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
    use crate::simulation::common::*;

    #[test]
    fn genome_grouping() {
        let chemistry_key = "cheese".to_string();
        let chemistry = get_chemistry_by_key(
            &chemistry_key,
            PlaceUnitsMethod::SimpleDrop { attributes: None },
        );
        let gm = GeneticManifest::new();
        let cm = chemistry.get_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let settings = SimpleExperimentSettings {
            cull_strategy: CullStrategy::WorstFirst,
            fitness_calculation_key: "total_cheese_consumed".to_string(),
            num_genomes: 10,
            sim_settings: ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                iterations: 20,
                default_unit_resources: vec![("cheese", 200)],
                default_unit_attr: vec![],
            },

            iterations: 10,
            alteration_set: alterations::default_alterations(),
            genetic_manifest: gm.clone(),
            sensor_manifest: sm.clone(),
            chemistry_key,
        };

        let mut exp = SimpleExperiment::new(settings);
        exp.seed_random_genomes(10);

        assert_eq!(exp.genome_entries.len(), 10);
    }

    #[test]
    fn test_partition_into_groups() {
        let chemistry_key = "cheese".to_string();
        let chemistry = get_chemistry_by_key(
            &chemistry_key,
            PlaceUnitsMethod::SimpleDrop { attributes: None },
        );
        let gm = GeneticManifest::new();
        let cm = chemistry.get_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let settings = SimpleExperimentSettings {
            cull_strategy: CullStrategy::WorstFirst,
            fitness_calculation_key: "total_cheese_consumed".to_string(),
            num_genomes: 10,
            sim_settings: ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                iterations: 20,
                default_unit_resources: vec![("cheese", 200)],
                default_unit_attr: vec![],
            },

            iterations: 10,
            genetic_manifest: gm.clone(),
            sensor_manifest: sm.clone(),
            chemistry_key,
            alteration_set: alterations::default_alterations(),
        };

        let mut exp = SimpleExperiment::new(settings);
        exp.seed_random_genomes(10);

        let groups = exp.partition_into_groups();
        println!("groups: {:?}", groups);
    }

    #[test]
    fn test_random_genome_generation() {
        let chemistry_key = "cheese".to_string();
        let chemistry = get_chemistry_by_key(&chemistry_key, PlaceUnitsMethod::Skip);
        let gm = GeneticManifest::new();
        let cm = chemistry.get_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);
        let vals1 = random_genome_of_length(100);

        assert_eq!(vals1.len(), 100);
        let genome1 = FramedGenomeParser::parse(vals1, cm.clone(), sm.clone(), gm.clone());
        print!("random genome: {}\n", genome1.display(&sm, &cm, &gm));
    }

    #[test]
    fn test_adjust_rank() {
        let chemistry_key = "cheese".to_string();
        let chemistry = get_chemistry_by_key(
            &chemistry_key,
            PlaceUnitsMethod::SimpleDrop { attributes: None },
        );
        let gm = GeneticManifest::new();
        let cm = chemistry.get_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        //assert_eq!(vals1.len(), 100);
        //let genome1 = FramedGenomeParser::parse(vals1, cm.clone(), sm.clone(), gm.clone());
        //print!("random genome: {}\n", genome1.display(&sm, &cm, &gm));

        let settings = SimpleExperimentSettings {
            cull_strategy: CullStrategy::WorstFirst,
            fitness_calculation_key: "total_cheese_consumed".to_string(),
            num_genomes: 10,
            sim_settings: ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                iterations: 20,
                default_unit_resources: vec![("cheese", 200)],
                default_unit_attr: vec![],
            },

            iterations: 10,
            genetic_manifest: gm.clone(),
            sensor_manifest: sm.clone(),
            chemistry_key,
            alteration_set: alterations::default_alterations(),
        };

        let mut exp = SimpleExperiment::new(settings);
        exp.seed_random_genomes(4);
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
                genome_uid: 0,
                fitness_score: 200,
            },
            TrialResultItem {
                genome_idx: 1,
                sim_unit_entry_id: 1,
                genome_uid: 1,
                fitness_score: 125,
            },
            TrialResultItem {
                genome_idx: 2,
                sim_unit_entry_id: 2,
                genome_uid: 2,
                fitness_score: 100,
            },
        ];

        exp.adjust_ranks_based_on_result(&fitness_result);

        assert_eq!(
            collect_max_fitnesses(&exp),
            vec![Some(200), Some(300), Some(400), Some(800)]
        );
        assert_eq!(collect_ranks(&exp), vec![2, 1, 0, 1]);
    }
    #[test]
    fn test_pull_genome() {
        let chemistry_key = "cheese".to_string();
        let chemistry = get_chemistry_by_key(
            &chemistry_key,
            PlaceUnitsMethod::SimpleDrop { attributes: None },
        );
        let gm = GeneticManifest::new();
        let cm = chemistry.get_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let settings = SimpleExperimentSettings {
            num_genomes: 10,
            cull_strategy: CullStrategy::WorstFirst,
            fitness_calculation_key: "total_cheese_consumed".to_string(),
            sim_settings: ExperimentSimSettings {
                num_simulation_ticks: 10,
                grid_size: (10, 10),
                num_genomes_per_sim: 2,
                iterations: 20,
                default_unit_resources: vec![("cheese", 200)],
                default_unit_attr: vec![],
            },

            iterations: 10,
            genetic_manifest: gm.clone(),
            sensor_manifest: sm.clone(),
            chemistry_key,
            alteration_set: alterations::default_alterations(),
        };

        let mut exp = SimpleExperiment::new(settings);
        exp.seed_random_genomes(4);
        exp.genome_entries[0].current_rank_score = 1;
        exp.genome_entries[0].max_fitness_metric = Some(175);
        exp.genome_entries[1].current_rank_score = 2;
        exp.genome_entries[1].max_fitness_metric = Some(300);
        exp.genome_entries[2].current_rank_score = 0;
        exp.genome_entries[2].max_fitness_metric = Some(50);
        exp.genome_entries[3].current_rank_score = 3;
        exp.genome_entries[3].max_fitness_metric = Some(800);
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
    genome_idx: UnitEntryId,
    genome_uid: ExperimentGenomeUid,
    fitness_score: FitnessScore,
}

impl Debug for TrialResultItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "TrialResult(uid: {}, score: {})",
            self.genome_uid, self.fitness_score
        )
    }
}
