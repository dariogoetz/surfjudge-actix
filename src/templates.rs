use crate::configuration::CONFIG;

use anyhow::Result;
use tera::Tera;

pub type Templates = Tera;
pub type Context = tera::Context;

pub async fn get_templates() -> Result<Tera> {
    let tera = tera::Tera::new(&format!("{}/*", &CONFIG.template_dir))?;

    Ok(tera)
}
