use crate::commands::log::format_commit_entry;
use crate::error::Result;
use crate::repo::Repo;

pub fn execute() -> Result<()> {
    let repo = Repo::find()?;
    let all_commits = repo.get_all_commits()?;

    for (id, commit) in all_commits {
        println!("{}", format_commit_entry(&id, &commit));
    }

    Ok(())
}
