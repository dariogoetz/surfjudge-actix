use crate::database::Pool;
use crate::models::result::Result;
use crate::models::user::User;
use crate::models::score::Score;
use crate::score_computation::{ResultComputation, Default};

pub struct PreliminaryResult {}

impl PreliminaryResult {
    pub async fn by_heat_id(
        db: &Pool,
        heat_id: u32,
    ) -> anyhow::Result<Vec<Result>> {
        let judges = User::find_by_judge_assignments(db, heat_id, false)
            .await?;
        let scores = Score::find_by_heat(db, heat_id)
            .await?;

        let results = Default::compute_results(heat_id as i32, &judges, &scores);

        Ok(results)
    }
}
