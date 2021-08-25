use crate::models::{user::User, score::Score, result::{Result, WaveScore}};

pub trait ResultComputation {
    fn compute_results(judges: &[User], scores: &[Score]) -> Vec<Result>;
}


// TODO: define an errortype for this, e.g. if not all judges gave a score
fn compute_individual_score(judges: &[User], scores: &[Score]) -> WaveScore {
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
