use super::{round_prec, ResultComputation, PRECISION};

use crate::models::result::{Result, WaveScore};

use std::collections::HashMap;

pub struct RSLHeat {}

impl ResultComputation for RSLHeat {
    fn process_wave_scores(
        &self,
        heat_id: i32,
        wave_scores: &Vec<(i32, i32, Option<WaveScore>)>,
    ) -> Vec<Result> {
        // group scores by wave
        let scores_by_wave = wave_scores.iter().fold(
            HashMap::<i32, Vec<&WaveScore>>::new(),
            |mut acc, (_, wave, wave_score)| {
                if let Some(wave_score) = wave_score {
                    acc.entry(*wave).or_insert(Vec::new()).push(wave_score);
                }
                acc
            },
        );

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

        // determine best score
        let mut total_scores_by_surfer: HashMap<i32, f64> = HashMap::new();
        for (_, scores) in scores_by_wave.iter() {
            let best_score = scores.iter().max_by(|s1, s2| {
                round_prec(s1.score, PRECISION)
                    .partial_cmp(&(round_prec(s2.score, PRECISION)))
                    .unwrap()
            });
            if let Some(best_score) = best_score {
                scores.iter().for_each(|s| {
                    let e = total_scores_by_surfer.entry(s.surfer_id).or_insert(0.0);
                    if round_prec(s.score, PRECISION) == round_prec(best_score.score, PRECISION) {
                        *e += 1.0;
                    }
                })
            }
        }
        let mut ranking_scores: Vec<(i32, f64)> = total_scores_by_surfer.into_iter().collect();
        ranking_scores.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(&s1).unwrap());

        // if two surfers have exactily the same scores, they should have the same placing
        let mut results = Vec::new();
        let mut place: i32 = 0;
        let mut prev_place = 0;
        let mut prev_total_score: Option<&f64> = None;
        for (idx, (surfer_id, total_score)) in ranking_scores.iter().enumerate() {
            if let Some(prev) = prev_total_score {
                if *prev == *total_score {
                    place = prev_place;
                } else {
                    prev_place = idx as i32;
                    place = idx as i32;
                }
            }
            prev_total_score = Some(total_score);

            let wave_scores: Vec<WaveScore> = scores_by_surfer
                .get(&surfer_id)
                .unwrap()
                .iter()
                .map(|&s| s.clone())
                .collect();

            results.push(Result {
                surfer_id: *surfer_id,
                total_score: *total_score,
                heat_id,
                place,
                wave_scores,
                heat: None,
                surfer: None,
            });
        }
        results
    }
}
