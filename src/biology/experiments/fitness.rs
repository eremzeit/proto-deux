use crate::simulation::{
    common::{ChemistryManifest, UnitManifest},
    fitness::FitnessScore,
    SimCell,
};

use super::types::{GenomeEntryId, TrialResultItem};

pub type ExperimentFitnessRank = usize;

#[derive(Clone)]
pub enum FitnessRankAdjustmentMethod {
    Absolute,

    // the winner's score increases by (rank_diff * pct_jump).max(min_jump)
    Incremental { pct_jump: f32, min_jump: usize },
}

pub fn calculate_new_fitness_ranks(
    fitness_result: &Vec<(TrialResultItem, ExperimentFitnessRank)>,
    method: &FitnessRankAdjustmentMethod,
) -> Vec<(TrialResultItem, ExperimentFitnessRank)> {
    let mut fitness_result = fitness_result.clone();
    fitness_result.sort_by_cached_key(|x| x.0.fitness_score);

    // println!("processing fitness result: {:?}", fitness_result);

    for i in 0..fitness_result.len() {
        // for (result_rank, fitness_result_item) in fitness_result.iter().enumerate() {
        let trial_result = &fitness_result[i].0;
        let mut our_rank_score = fitness_result[i].1;

        let our_genome_uid = trial_result.experiment_genome_uid;
        let our_fitness_score = trial_result.fitness_score;

        // for each entry that we beat
        for j in (0..i) {
            let losers_uid = fitness_result[j].0.experiment_genome_uid;
            let losers_rank_score = fitness_result[j].1;
            let losers_fitness_score = fitness_result[j].0.fitness_score;

            // let mut our_new_rank_score = self.genome_entries[our_genome_idx].current_rank_score;
            assert!(our_fitness_score > losers_fitness_score);

            our_rank_score = adjust_winners_rank(our_rank_score, losers_rank_score, &method);
            fitness_result[i].1 = our_rank_score;
        }
    }
    fitness_result
}

pub fn adjust_winners_rank(
    winner_rank: ExperimentFitnessRank,
    loser_rank: ExperimentFitnessRank,
    method: &FitnessRankAdjustmentMethod,
) -> ExperimentFitnessRank {
    // if its not an upset, don't change rank
    if winner_rank > loser_rank {
        return winner_rank;
    }

    match method {
        FitnessRankAdjustmentMethod::Absolute => loser_rank + 1,
        FitnessRankAdjustmentMethod::Incremental { pct_jump, min_jump } => {
            let diff = loser_rank - winner_rank + 1;
            println!("diff: {}", diff);
            let jump_amount =
                ((diff as f32 * pct_jump).ceil() as ExperimentFitnessRank).max(*min_jump);
            println!("jump amount : {}", jump_amount);

            winner_rank + jump_amount
        }
    }
}

pub fn normalize_ranks(mut ranks: &mut Vec<(GenomeEntryId, ExperimentFitnessRank)>) {
    ranks.sort_by_cached_key(|(genome_id, score)| *score);

    let mut current_rank_tally = 0;
    let mut prev_rank = None;
    for i in (0..ranks.len()) {
        let _current = ranks[i].1;
        if prev_rank != None && prev_rank != Some(_current) {
            current_rank_tally += 1;
        }

        ranks[i].1 = current_rank_tally;

        prev_rank = Some(_current);
    }
}

pub mod tests {
    use super::{adjust_winners_rank, normalize_ranks, FitnessRankAdjustmentMethod};

    #[test]
    pub fn test_normalize_ranks() {
        let mut ranks = vec![(0, 2), (1, 5), (2, 5), (3, 5), (4, 10), (5, 7), (6, 2)];
        normalize_ranks(&mut ranks);

        assert_eq!(
            ranks,
            vec![(0, 0), (6, 0), (1, 1), (2, 1), (3, 1), (5, 2), (4, 3)]
        )
    }

    #[test]
    pub fn test_adjust_rank() {
        assert_eq!(
            adjust_winners_rank(2, 5, &FitnessRankAdjustmentMethod::Absolute),
            6
        );

        assert_eq!(
            adjust_winners_rank(7, 5, &FitnessRankAdjustmentMethod::Absolute),
            7
        );
        assert_eq!(
            adjust_winners_rank(
                1,
                9,
                &FitnessRankAdjustmentMethod::Incremental {
                    pct_jump: 0.25,
                    min_jump: 1
                }
            ),
            4
        );

        assert_eq!(
            adjust_winners_rank(
                1,
                9,
                &FitnessRankAdjustmentMethod::Incremental {
                    pct_jump: 0.25,
                    min_jump: 4
                }
            ),
            5
        );
    }
}
