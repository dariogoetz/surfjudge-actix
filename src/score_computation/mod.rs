use crate::logging::LOG;
use crate::models::{
    result::{Result, WaveScore},
    score::Score,
    user::User,
};

use slog::info;
use std::collections::HashMap;

pub trait ResultComputation {
    fn compute_results(heat_id: i32, judges: &[User], scores: &[Score]) -> Vec<Result>;
}


fn round_prec(val: f64, prec: i32) -> f64 {
    let d = (10.0_f64).powi(prec);
    (val * d).round() / d
}

// TODO: define an errortype for this, e.g. if not all judges gave a score
fn compute_individual_score(
    surfer_id: i32,
    wave: i32,
    judges: &[User],
    scores: &[&Score],
) -> WaveScore {
    info!(LOG, "Scores for surfer {} in wave {}: {:?}", surfer_id, wave, scores);
    // TODO: check if all judges provided a score
    // TODO: sort scores by score
    // TODO: remove best and worst score
    // TODO: take mean of remaining scores
    let score = scores.iter().map(|s| s.score).sum::<f64>() / scores.len() as f64;
    info!(LOG, "Averaged score: {}", score);
    WaveScore {
        surfer_id,
        wave,
        score,
    }
}

pub struct Default {}

impl ResultComputation for Default {
    fn compute_results(heat_id: i32, judges: &[User], scores: &[Score]) -> Vec<Result> {
        let n_best_waves = 2;

        // divide scores by wave id and surfer
        let grouped_scores =
            scores
                .iter()
                .fold(HashMap::<(i32, i32), Vec<&Score>>::new(), |mut acc, s| {
                    acc.entry((s.surfer_id, s.wave))
                        .or_insert(Vec::new())
                        .push(s);
                    acc
                });

        // compute individual results per wave and surfer (use compute_individual_score)
        let wave_scores: Vec<(i32, i32, WaveScore)> = grouped_scores
            .iter()
            .map(|((surfer_id, wave), wave_scores)| {
                (
                    *surfer_id,
                    *wave,
                    compute_individual_score(*surfer_id, *wave, judges, wave_scores),
                )
            })
            .collect();

        // collect wave scores by surfer
        let mut surfer_scores = wave_scores.iter().fold(
            HashMap::<i32, Vec<&WaveScore>>::new(),
            |mut acc, (surfer_id, _, wave_score)| {
                acc.entry(*surfer_id).or_insert(Vec::new()).push(wave_score);
                acc
            },
        );

        // determine best n waves by surfer
        let mut total_scores: Vec<(i32, Vec<f64>)> = surfer_scores
            .iter_mut()
            .map(|(&surfer_id, wave_scores)| {
                // sort waves by score
                wave_scores.sort_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap());

                // only take best n waves
                let total_score: f64 =
                    wave_scores.iter().take(n_best_waves).map(|s| round_prec(s.score, 5)).sum();

                let other_scores: Vec<f64> = wave_scores.iter().skip(n_best_waves).map(|s| round_prec(s.score, 5)).collect();
                let mut rank_scores = Vec::new();
                rank_scores.push(total_score);
                rank_scores.extend(other_scores);

                (surfer_id, rank_scores)
            })
            .collect();

        // sort surfer scores lexicographically by total score and then all other scores
        total_scores.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(s1).unwrap());

        // if two surfers have exactily the same scores, they should have the same placing
        let mut results = Vec::new();
        let mut place: i32 = 0;
        let mut prev_place = 0;
        let mut prev_rank_scores: Option<&Vec<f64>> = None;
        for (idx, (surfer_id, rank_scores)) in total_scores.iter().enumerate() {
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

            results.push(Result {
                surfer_id: *surfer_id,
                heat_id,
                place,
                total_score,
                wave_scores: surfer_scores.get(&surfer_id).unwrap().iter().map(|&s| s.clone()).collect(),
                heat: None,
                surfer: None,
            });
            info!(LOG, "Surfer {}, score {:?}, place {}", surfer_id, rank_scores, place);
        }

        results
    }
}

pub struct RSL {}

impl ResultComputation for RSL {
    fn compute_results(heat_id: i32, judges: &[User], scores: &[Score]) -> Vec<Result> {
        Vec::new()
    }
}
