use crate::configuration::CONFIG;

use tera::Tera;
use anyhow::Result;

pub type Templates = Tera;
pub type Context = tera::Context;

pub async fn get_templates() -> Result<Tera> {
    let tera = tera::Tera::new(
        &format!("{}/*", &CONFIG.template_dir)
    )?;

    Ok(tera)
}
