use crate::database::Pool;
use crate::models::result::Result;
use crate::models::user::User;
use crate::models::score::Score;
use crate::models::heat::{Heat, HeatType};
use crate::score_computation::{default_heat::DefaultHeat, rsl_heat::RSLHeat, compute_results};

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
        let heat = Heat::find_by_id(db, heat_id, false)
            .await?;
        let results = Result::find_by_heat_id(db, heat_id, false)
            .await?;

        if heat.is_none() {
           return Ok(Vec::new())
        }
        let heat = heat.unwrap();

        let results = match heat.heat_type {
            HeatType::Standard => compute_results(heat_id as i32, &judges, &scores, &results, &DefaultHeat::default()),
            HeatType::Call => compute_results(heat_id as i32, &judges, &scores, &results, &RSLHeat{}),
        };

        Ok(results)
    }
}
