use super::{
    types::{CullStrategy, ExperimentGenomeUid, GenomeEntryId, GenomeExperimentEntry},
    variants::multi_pool::types::FitnessCycleStrategy,
};
use rand::Rng;

#[derive(Clone, Debug)]
pub struct GenomeEntryInfo {
    pub id: GenomeEntryId,
    pub uid: ExperimentGenomeUid,
    pub fitness_rank: usize,
}

pub fn partition_genomes_into_subset_groups(
    genomes: Vec<GenomeEntryInfo>,
    group_size: usize,
    subset_pct_size: f32,
    scramble_pct: f32,
) -> Vec<Vec<GenomeEntryInfo>> {
    let total_in_subset = (genomes.len() as f32 * subset_pct_size) as usize;

    let mut subset_genomes = select_random_subset_into(genomes.clone(), total_in_subset);

    subset_genomes.sort_by_cached_key(|entry| entry.fitness_rank);
    partition_groups(subset_genomes, group_size)
}

pub fn select_random_subset_into(
    mut values: Vec<GenomeEntryInfo>,
    subset_size: usize,
) -> Vec<GenomeEntryInfo> {
    let mut result = vec![];

    let mut rng = rand::thread_rng();

    while result.len() < subset_size {
        let selected_idx = rng.gen_range(0..values.len());
        result.push(values.remove(selected_idx));
    }

    result
}

pub fn partition_genomes_into_exaustive_groups(
    genomes: Vec<GenomeEntryInfo>,
    group_size: usize,
) -> Vec<Vec<GenomeEntryInfo>> {
    let num_genomes = genomes.len();

    let mut entries_by_fitness = genomes.clone();
    entries_by_fitness.sort_by_cached_key(|entry| entry.fitness_rank);

    partition_groups(entries_by_fitness, group_size)
}

pub fn scramble_groups<T>(
    mut groups: Vec<Vec<T>>,
    fitness_cycle_strategy: &FitnessCycleStrategy,
) -> Vec<Vec<T>> {
    match fitness_cycle_strategy {
        FitnessCycleStrategy::Exaustive { group_scramble_pct } => {
            _scramble_groups(groups, *group_scramble_pct)
        }
        FitnessCycleStrategy::RandomSubset {
            percent_of_genomes,
            group_scramble_pct,
        } => _scramble_groups(groups, *group_scramble_pct),
    }
}

pub fn _scramble_groups<T>(mut groups: Vec<Vec<T>>, pct_scramble: f32) -> Vec<Vec<T>> {
    if groups.len() == 0 || groups[0].len() == 0 || pct_scramble == 0.0 {
        return groups;
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for i in 0..groups.len() {
        let to_shuffle: usize = (groups[i].len() as f32 * pct_scramble).round() as usize;

        for count in 0..to_shuffle {
            let dest_group = rng.gen_range(0..groups.len());

            let src_group_length = groups[i].len();
            let removed = groups[i].remove(rng.gen_range(0..src_group_length));
            groups[dest_group].push(removed);

            let dest_group_length = groups[dest_group].len();
            let removed = groups[dest_group].remove(rng.gen_range(0..dest_group_length));
            groups[i].push(removed);
        }
    }

    groups
}

pub fn partition_groups<T: Clone>(items: Vec<T>, group_size: usize) -> Vec<Vec<T>> {
    let item_count = items.len();
    let mut partitions = vec![];
    let mut num_groups = item_count / group_size;

    let mut last_group_is_uneven = false;
    if item_count % group_size > 0 {
        num_groups += 1;
        last_group_is_uneven = true;
    }

    let mut item_idx = 0;
    for group_idx in (0..num_groups) {
        let mut group = vec![];

        if last_group_is_uneven && group_idx == num_groups - 1 {
            for i in (0..group_size) {
                // count backwards from the end of the genome list. ie. some genomes will get included in two groups
                let item_idx = item_count - 1 - i;
                group.insert(0, items[item_idx].clone());
            }
        } else {
            for i in (0..group_size) {
                group.push(items[item_idx].clone());
                item_idx += 1;
            }
        }

        partitions.push(group);
    }
    //let flattened = partitions.iter().flatten().collect::<Vec<_>>();
    //println!("flattened groups: {:?}", flattened.len());
    partitions
}

/**
 * items should already be sorted before passing as argument
 */
pub fn partition_into_thirds<T: Clone>(items: &Vec<T>) -> Vec<Vec<T>> {
    let count = items.len();
    let group_size = count / 3;
    let group_size_remainder = count % 3;

    /*
    11 / 3 = 3
    11 % 3 = 2
    ....
    ....
    ...
    */

    let mut groups = vec![];
    let mut idx = 0;
    for group_idx in 0..3 {
        let mut group = vec![];

        let group_size_adj = if idx < group_size_remainder {
            group_size + 1
        } else {
            group_size
        };

        for sub_i in 0..group_size_adj {
            group.push(items[idx].clone());
            idx += 1;
        }

        groups.push(group);
    }

    groups
}

/**
 * Genomes are sorted by increasing fitness
 */
pub fn partition_into_groups(
    genomes: Vec<GenomeEntryInfo>,
    fitness_cycle_strategy: &FitnessCycleStrategy,
    group_size: usize,
) -> Vec<Vec<GenomeEntryInfo>> {
    match fitness_cycle_strategy {
        FitnessCycleStrategy::Exaustive { group_scramble_pct } => {
            partition_genomes_into_exaustive_groups(genomes, group_size)
        }
        FitnessCycleStrategy::RandomSubset {
            percent_of_genomes,
            group_scramble_pct,
        } => partition_genomes_into_subset_groups(
            genomes,
            group_size,
            *percent_of_genomes,
            *group_scramble_pct,
        ),
    }
}

pub fn cull_worst_first(
    genomes: &mut Vec<GenomeExperimentEntry>,
    percent_cull: f32,
    num_genomes: usize,
) {
    let target_count = (num_genomes as f32 * percent_cull) as usize;
    let to_remove = genomes.len() - target_count;

    let mut by_rank = genomes
        .iter()
        .filter(|entry| entry.max_fitness_metric.is_some())
        .enumerate()
        .map(|(i, entry)| {
            (
                i,
                entry.uid,
                entry.current_rank_score,
                entry.max_fitness_metric,
                entry.num_evaluations,
            )
        })
        .collect::<Vec<_>>()
        .clone();
    by_rank.sort_by_cached_key(|entry| entry.2);

    let mut uids_to_remove = vec![];
    while uids_to_remove.len() < to_remove && by_rank.len() > 0 {
        let item = by_rank.remove(0);
        uids_to_remove.push(item.1);
    }

    // explog!("Culling genomes with uids: {:?}", &uids_to_remove);
    for uid in uids_to_remove {
        let idx = (0..genomes.len())
            .find(|(i)| genomes[*i].uid == uid)
            .unwrap();

        genomes.remove(idx);
    }
}

pub fn cull_percent_in_tercile(
    genomes: &mut Vec<GenomeExperimentEntry>,
    percent_by_tercile: [f32; 3],
    num_genomes: usize,
) {
    let mut rng = rand::thread_rng();
    let mut by_rank = genomes
        .iter()
        .filter(|entry| entry.max_fitness_metric.is_some())
        .enumerate()
        .map(|(i, entry)| {
            (
                i,
                entry.uid,
                entry.current_rank_score,
                entry.max_fitness_metric,
                entry.num_evaluations,
            )
        })
        .collect::<Vec<_>>()
        .clone();
    by_rank.sort_by_cached_key(|entry| entry.2);

    let mut groups = partition_into_thirds(&by_rank);

    let mut to_remove_groups = vec![];
    for i in 0..3 {
        let mut uids_to_remove = vec![];
        let to_remove_count = (groups[i].len() as f32 * percent_by_tercile[i]).round() as usize;
        while uids_to_remove.len() < to_remove_count && groups[i].len() > 0 {
            let to_remove_idx = rng.gen_range(0..groups[i].len());
            let item = groups[i].remove(to_remove_idx);
            uids_to_remove.push(item.1);
        }

        to_remove_groups.push(uids_to_remove);
    }

    let uids_to_remove = to_remove_groups.iter().flatten().collect::<Vec<_>>();

    // explog!("Culling genomes with uids: {:?}", &uids_to_remove);
    for uid in uids_to_remove {
        let idx = (0..genomes.len())
            .find(|(i)| genomes[*i].uid == *uid)
            .unwrap();

        genomes.remove(idx);
    }
}

pub fn cull_genomes(
    genomes: &mut Vec<GenomeExperimentEntry>,
    method: &CullStrategy,
    total_num_genomes: usize,
) {
    match method {
        CullStrategy::WorstFirst { percent } => {
            cull_worst_first(genomes, *percent, total_num_genomes)
        }
        CullStrategy::RandomTiers {
            percent_per_tercile,
        } => {
            panic!("this needs testing");
            cull_percent_in_tercile(genomes, percent_per_tercile.clone(), total_num_genomes)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::biology::experiments::util::{_scramble_groups, partition_into_thirds};

    use super::{partition_groups, scramble_groups};

    #[test]
    pub fn test_scramble_groups() {
        let input = vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9, 10]];

        // non-deterministic sanity check
        for i in 0..5 {
            let result = _scramble_groups(input.clone(), 0.10);
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].len(), 5);
            assert_eq!(result[1].len(), 5);
        }
    }

    #[test]
    pub fn test_partition_groups() {
        //simple
        assert_eq!(
            partition_groups(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 5),
            vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9, 10]]
        );

        //simple
        assert_eq!(
            partition_groups(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13], 5),
            vec![
                vec![1, 2, 3, 4, 5],
                vec![6, 7, 8, 9, 10],
                vec![9, 10, 11, 12, 13]
            ]
        );
    }

    #[test]
    pub fn test_partition_into_quartiles() {
        assert_eq!(
            partition_into_thirds(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12],],
        );

        assert_eq!(
            partition_into_thirds(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]),
            vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9], vec![10, 11, 12, 13],],
        );
    }
}
