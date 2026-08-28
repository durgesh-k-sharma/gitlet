use crate::error::Result;
use crate::repo::Repo;
use std::env;

pub fn execute() -> Result<()> {
    let current_dir = env::current_dir()?;
    Repo::init(&current_dir)?;
    Ok(())
}
