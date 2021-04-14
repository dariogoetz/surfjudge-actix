use crate::configuration::CONFIG;
use actix_files::NamedFile;
use std::path::PathBuf;

use actix_web::Result;

pub async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open::<PathBuf>(
        format!("{}/index.html", &CONFIG.template_dir).into(),
    )?)
}

pub async fn index_judging() -> Result<NamedFile> {
    Ok(NamedFile::open::<PathBuf>(
        format!("{}/index-judging.html", &CONFIG.template_dir).into(),
    )?)
}

pub async fn index_admin() -> Result<NamedFile> {
    Ok(NamedFile::open::<PathBuf>(
        format!("{}/index-admin.html", &CONFIG.template_dir).into(),
    )?)
}
