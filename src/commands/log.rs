use crate::error::Result;
use crate::models::Commit;
use crate::repo::Repo;

pub fn format_commit_entry(id: &str, commit: &Commit) -> String {
    let mut out = String::new();
    out.push_str("===\n");
    out.push_str(&format!("commit {}\n", id));
    if commit.is_merge() {
        let p1 = if commit.parents[0].len() >= 7 {
            &commit.parents[0][..7]
        } else {
            &commit.parents[0]
        };
        let p2 = if commit.parents[1].len() >= 7 {
            &commit.parents[1][..7]
        } else {
            &commit.parents[1]
        };
        out.push_str(&format!("Merge: {} {}\n", p1, p2));
    }
    out.push_str(&format!("Date: {}\n", commit.timestamp));
    out.push_str(&format!("{}\n", commit.message));
    out
}

pub fn execute() -> Result<()> {
    let repo = Repo::find()?;
    let mut current_id = repo.get_head_commit_id()?;

    loop {
        let commit = repo.read_commit(&current_id)?;
        println!("{}", format_commit_entry(&current_id, &commit));

        if commit.parents.is_empty() {
            break;
        }
        current_id = commit.parents[0].clone();
    }

    Ok(())
}
