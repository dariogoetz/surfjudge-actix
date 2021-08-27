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
fn compute_individual_score(judges: &[User], scores: &[&Score]) -> WaveScore {
    info!(LOG, "{:?}", scores);
    // TODO: check if all judges provided a score
    // TODO: sort scores by score
    // TODO: remove best and worst score
    // TODO: take mean of remaining scores
    WaveScore {
        surfer_id: 0,
        wave: 0,
        score: 1.0,
    }
}

pub struct Default {}

impl ResultComputation for Default {
    fn compute_results(judges: &[User], scores: &[Score]) -> Vec<Result> {
        // TODO: divide scores by wave id and surfer
        let grouped_scores =
            scores
                .iter()
                .fold(HashMap::<(i32, i32), Vec<&Score>>::new(), |mut acc, s| {
                    acc.entry((s.wave, s.surfer_id)).or_insert(vec![]).push(s);
                    acc
                });
        info!(LOG, "Grouped scores: {:?}", grouped_scores);
        let wave_scores: Vec<(i32, i32, WaveScore)> = grouped_scores.iter().map(|((wave, surfer_id), wave_scores)| {
            (*wave, *surfer_id, compute_individual_score(judges, wave_scores))
        }).collect();
        // TODO: compute individual results per wave and surfer (use compute_individual_score)
        // TODO: determine best n waves by surfer
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
