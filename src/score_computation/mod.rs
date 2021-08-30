use crate::logging::LOG;
use crate::models::{
    result::{Result, WaveScore},
    score::Score,
    user::User,
};

use slog::info;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub trait ResultComputation {
    fn process_wave_scores(
        &self,
        heat_id: i32,
        judge_ids: &[User],
        wave_scores: &Vec<(i32, i32, Option<WaveScore>)>,
    ) -> Vec<Result>;
}

const PRECISION: i32 = 5;

pub fn compute_results(
    heat_id: i32,
    judges: &[User],
    scores: &[Score],
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

    score_processor.process_wave_scores(heat_id, judges, &wave_scores)
}

fn round_prec(val: f64, prec: i32) -> f64 {
    let d = (10.0_f64).powi(prec);
    (val * d).round() / d
}

fn compute_individual_score(
    surfer_id: i32,
    wave: i32,
    judge_ids: &HashSet<i32>,
    scores: &[&Score],
) -> Option<WaveScore> {
    info!(
        LOG,
        "Scores for surfer {} in wave {}: {:?}", surfer_id, wave, scores
    );
    let score_judges: HashSet<i32> = HashSet::from_iter(scores.iter().map(|s| s.judge_id));
    if (*judge_ids != score_judges) || (judge_ids.len() != scores.len()) {
        info!(
            LOG,
            "Not all judges provided scores for surfer {}, wave {}", surfer_id, wave
        );
        return None;
    }

    // sort scores by score
    let mut ranked_scores: Vec<&Score> = scores.to_vec();
    ranked_scores.sort_by(|s1, s2| s1.score.partial_cmp(&s2.score).unwrap());

    let score = if scores.len() > 4 {
        // remove best and worst score
        // take mean of remaining scores
        ranked_scores.iter().skip(1).take(scores.len() - 2).map(|s| s.score).sum::<f64>() / (scores.len() - 2) as f64
    } else {
        ranked_scores.iter().map(|s| s.score).sum::<f64>() / scores.len() as f64
    };
    let score = round_prec(score, PRECISION);
    info!(LOG, "Averaged score for surfer {}, wave {}: {}", surfer_id, wave, score);
    Some(WaveScore {
        surfer_id,
        wave,
        score,
    })
}

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
        judges: &[User],
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
            info!(
                LOG,
                "Surfer {}, score {:?}, place {}", surfer_id, rank_scores, place
            );
        }

        results
    }
}
pub struct RSLHeat {}

impl ResultComputation for RSLHeat {
    fn process_wave_scores(
        &self,
        heat_id: i32,
        judges: &[User],
        wave_scores: &Vec<(i32, i32, Option<WaveScore>)>,
    ) -> Vec<Result> {
        Vec::new()
    }
}
