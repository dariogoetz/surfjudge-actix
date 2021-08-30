use super::{ResultComputation, round_prec, PRECISION};

use crate::models::result::{Result, WaveScore};

use std::collections::HashMap;

pub struct DefaultHeat {
    pub n_best_waves: usize,
}

impl Default for DefaultHeat {
    fn default() -> Self {
        DefaultHeat { n_best_waves: 2 }
    }
}

impl ResultComputation for DefaultHeat {
    fn process_wave_scores(
        &self,
        heat_id: i32,
        wave_scores: &Vec<(i32, i32, Option<WaveScore>)>,
    ) -> Vec<Result> {
        // collect wave scores by surfer
        let mut scores_by_surfer = wave_scores.iter().fold(
            HashMap::<i32, Vec<&WaveScore>>::new(),
            |mut acc, (surfer_id, _, wave_score)| {
                if let Some(wave_score) = wave_score {
                    acc.entry(*surfer_id).or_insert(Vec::new()).push(wave_score);
                }
                acc
            },
        );

        // sort surfer scores by wave number
        scores_by_surfer
            .iter_mut()
            .for_each(|(_, scores)| scores.sort_by(|s1, s2| s1.wave.cmp(&s2.wave)));

        // determine best n waves by surfer
        let mut ranking_scores: Vec<(i32, Vec<f64>)> = scores_by_surfer
            .iter()
            .map(|(&surfer_id, wave_scores)| {
                // make a copy of the vec for sorting by score
                let mut sorted_scores = wave_scores.clone();
                // sort waves by score
                sorted_scores.sort_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap());

                // only take best n waves
                let total_score: f64 = sorted_scores
                    .iter()
                    .take(self.n_best_waves)
                    .map(|s| round_prec(s.score, PRECISION))
                    .sum();

                let mut other_scores: Vec<f64> = sorted_scores
                    .iter()
                    .skip(self.n_best_waves)
                    .map(|s| round_prec(s.score, PRECISION))
                    .collect();
                other_scores.sort_by(|s1, s2| s2.partial_cmp(&s1).unwrap());
                let mut rank_scores = Vec::new();
                rank_scores.push(total_score);
                rank_scores.extend(other_scores);

                (surfer_id, rank_scores)
            })
            .collect();

        // sort surfer scores lexicographically by total score and then all other scores
        ranking_scores.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(s1).unwrap());

        // if two surfers have exactily the same scores, they should have the same placing
        let mut results = Vec::new();
        let mut place: i32 = 0;
        let mut prev_place = 0;
        let mut prev_rank_scores: Option<&Vec<f64>> = None;
        for (idx, (surfer_id, rank_scores)) in ranking_scores.iter().enumerate() {
            if let Some(prev) = prev_rank_scores {
                if *prev == *rank_scores {
                    place = prev_place;
                } else {
                    prev_place = idx as i32;
                    place = idx as i32;
                }
            }
            prev_rank_scores = Some(rank_scores);

            let mut total_score = 0.0;
            if rank_scores.len() > 0 {
                total_score = rank_scores[0];
            }
            let wave_scores: Vec<WaveScore> = scores_by_surfer
                .get(&surfer_id)
                .unwrap()
                .iter()
                .map(|&s| s.clone())
                .collect();

            results.push(Result {
                surfer_id: *surfer_id,
                heat_id,
                place,
                total_score,
                wave_scores,
                heat: None,
                surfer: None,
            });
        }

        results
    }
}
