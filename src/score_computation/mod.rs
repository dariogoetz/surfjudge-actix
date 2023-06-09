use crate::logging::LOG;
use crate::models::{
    result::{Result, WaveScore},
    score::Score,
    user::User,
};

use slog::debug;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub mod default_heat;
pub mod rsl_heat;

pub trait ResultComputation {
    fn process_wave_scores(
        &self,
        heat_id: i32,
        wave_scores: &Vec<(i32, i32, Option<WaveScore>)>,
    ) -> Vec<Result>;
}

const EPSILON: f64 = 1e-5;
const MIN_JUDGES_FOR_DROP: usize = 4;
const DROP_SCORES: usize = 1;

pub fn compute_results(
    heat_id: i32,
    judges: &[User],
    scores: &[Score],
    results: &[Result],
    score_processor: &impl ResultComputation,
) -> Vec<Result> {
    // set of judge_ids for filtering
    let judge_set: HashSet<i32> = HashSet::from_iter(judges.iter().map(|j| j.id));

    // divide scores by wave id and surfer (and filter relevant judges)
    let scores_grouped = scores
        .iter()
        .filter(|s| judge_set.contains(&s.judge_id))
        .fold(HashMap::<(i32, i32), Vec<&Score>>::new(), |mut acc, s| {
            acc.entry((s.surfer_id, s.wave))
                .or_insert(Vec::new())
                .push(s);
            acc
        });

    // compute individual results per wave and surfer (use compute_individual_score)
    let wave_scores: Vec<(i32, i32, Option<WaveScore>)> = scores_grouped
        .iter()
        .map(|((surfer_id, wave), individual_scores)| {
            (
                *surfer_id,
                *wave,
                compute_individual_score(*surfer_id, *wave, &judge_set, individual_scores),
            )
        })
        .collect();

    let mut preliminary_results = score_processor.process_wave_scores(heat_id, &wave_scores);

    let grouped_results = results
        .iter()
        .fold(HashMap::<i32, &Result>::new(), |mut acc, r| {
            acc.insert(r.surfer_id, r);
            acc
        });

    let grouped_pub_wave_scores =
        results
            .iter()
            .fold(HashMap::<(i32, i32), &WaveScore>::new(), |mut acc, r| {
                r.wave_scores.iter().for_each(|ws| {
                    acc.insert((ws.surfer_id, ws.wave), ws);
                });
                acc
            });
    preliminary_results.iter_mut().for_each(|pr| {
        if let Some(existing_result) = grouped_results.get(&pr.surfer_id) {
            if float_eq(existing_result.total_score, pr.total_score) {
                pr.published = true;
            }
        }
        pr.wave_scores.iter_mut().for_each(|ws| {
            if let Some(existing_wave_score) = grouped_pub_wave_scores.get(&(ws.surfer_id, ws.wave))
            {
                if float_eq(existing_wave_score.score, ws.score) {
                    ws.published = true;
                }
            }
        });
    });

    preliminary_results
}

fn float_eq(val1: f64, val2: f64) -> bool {
    (val1 - val2).abs() < EPSILON
}

fn compute_individual_score(
    surfer_id: i32,
    wave: i32,
    judge_ids: &HashSet<i32>,
    scores: &[&Score],
) -> Option<WaveScore> {
    let score_judges: HashSet<i32> = HashSet::from_iter(scores.iter().map(|s| s.judge_id));
    if (*judge_ids != score_judges) || (judge_ids.len() != scores.len()) {
        debug!(
            LOG,
            "Not all judges provided scores for surfer {}, wave {}", surfer_id, wave
        );
        return None;
    }

    // sort scores by score
    let mut ranked_scores: Vec<f64> = scores
        .iter()
        .filter(|s| !s.missed)
        .map(|s| s.score)
        .collect();

    if ranked_scores.len() == 0 {
        debug!(
            LOG,
            "All judges missed score for surfer {}, wave {}", surfer_id, wave
        );
        return None;
    }
    // fill missed scores with average of non-missed scores
    let n_missed = scores.len() - ranked_scores.len();
    let missed_substitute = ranked_scores.iter().sum::<f64>() / ranked_scores.len() as f64;
    ranked_scores.extend(vec![missed_substitute; n_missed]);

    // sort scores before removing first and last ones
    ranked_scores.sort_by(|s1, s2| s1.partial_cmp(&s2).unwrap());

    let score = if (scores.len() > MIN_JUDGES_FOR_DROP) && (ranked_scores.len() > 2 * DROP_SCORES) {
        let n = ranked_scores.len() - 2 * DROP_SCORES;

        // remove best and worst score
        // take mean of remaining scores
        ranked_scores
            .iter()
            .skip(DROP_SCORES)
            .take(n)
            .map(|s| s)
            .sum::<f64>()
            / n as f64
    } else {
        ranked_scores.iter().sum::<f64>() / scores.len() as f64
    };
    Some(WaveScore {
        surfer_id,
        wave,
        score,
        published: false,
    })
}
