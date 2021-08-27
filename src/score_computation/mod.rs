use crate::logging::LOG;
use crate::models::{
    result::{Result, WaveScore},
    score::Score,
    user::User,
};

use slog::info;
use std::collections::HashMap;

pub trait ResultComputation {
    fn compute_results(judges: &[User], scores: &[Score]) -> Vec<Result>;
}

// TODO: define an errortype for this, e.g. if not all judges gave a score
fn compute_individual_score(
    surfer_id: i32,
    wave: i32,
    judges: &[User],
    scores: &[&Score],
) -> WaveScore {
    info!(LOG, "{:?}", scores);
    // TODO: check if all judges provided a score
    // TODO: sort scores by score
    // TODO: remove best and worst score
    // TODO: take mean of remaining scores
    let score = scores.iter().map(|s| s.score).sum::<f64>() / scores.len() as f64;
    WaveScore {
        surfer_id,
        wave,
        score,
    }
}

pub struct Default {}

impl ResultComputation for Default {
    fn compute_results(judges: &[User], scores: &[Score]) -> Vec<Result> {
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
        info!(LOG, "Grouped scores: {:?}", grouped_scores);

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
        let surfer_scores = wave_scores.iter().fold(
            HashMap::<i32, Vec<&WaveScore>>::new(),
            |mut acc, (surfer_id, _, wave_score)| {
                acc.entry(*surfer_id).or_insert(Vec::new()).push(wave_score);
                acc
            },
        );

        // determine best n waves by surfer
        let best_n_scores: Vec<(i32, Vec<&WaveScore>)> = surfer_scores
            .into_iter()
            .map(|(surfer_id, mut wave_scores)| {
                // sort waves by score
                wave_scores.sort_by(|s1, s2| s2.score.partial_cmp(&s1.score).unwrap());

                // only take best n waves
                let best_scores: Vec<&WaveScore> =
                    wave_scores.into_iter().take(n_best_waves).collect();

                (surfer_id, best_scores)
            })
            .collect();
        info!(LOG, "Best 2 waves: {:?}", best_n_scores);

        // TODO: compute total scores and placings
        // TODO: generate results
        Vec::new()
    }
}

pub struct RSL {}

impl ResultComputation for RSL {
    fn compute_results(judges: &[User], scores: &[Score]) -> Vec<Result> {
        Vec::new()
    }
}
