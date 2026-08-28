use crate::error::Result;
use crate::repo::Repo;
use std::collections::{HashMap, HashSet, VecDeque};

/// Finds the Latest Common Ancestor (split point) between two commits in the DAG.
/// An LCA is a common ancestor `u` such that no other common ancestor `v` is a descendant of `u`.
pub fn find_split_point(repo: &Repo, current_id: &str, given_id: &str) -> Result<String> {
    if current_id == given_id {
        return Ok(current_id.to_string());
    }

    // 1. Collect all reachable ancestors of current_id with their graph depths/distances
    let (ancestors_current, dist_current) = collect_ancestors_with_depths(repo, current_id)?;

    // Fast-path: If given_id is an ancestor of current_id
    if ancestors_current.contains(given_id) {
        return Ok(given_id.to_string());
    }

    // 2. Collect all reachable ancestors of given_id
    let (ancestors_given, dist_given) = collect_ancestors_with_depths(repo, given_id)?;

    // Fast-path: If current_id is an ancestor of given_id
    if ancestors_given.contains(current_id) {
        return Ok(current_id.to_string());
    }

    // 3. Find common ancestors
    let common_ancestors: HashSet<String> = ancestors_current
        .intersection(&ancestors_given)
        .cloned()
        .collect();

    if common_ancestors.is_empty() {
        return Ok(current_id.to_string());
    }

    // 4. Filter out any common ancestor `u` that is an ancestor of another common ancestor `v`.
    // In other words, keep only maximal elements in the ancestor poset (true LCAs).
    let mut candidate_lcas: Vec<String> = Vec::new();
    for u in &common_ancestors {
        let mut is_older_ancestor = false;
        for v in &common_ancestors {
            if u != v {
                let (v_ancestors, _) = collect_ancestors_with_depths(repo, v)?;
                if v_ancestors.contains(u) {
                    is_older_ancestor = true;
                    break;
                }
            }
        }
        if !is_older_ancestor {
            candidate_lcas.push(u.clone());
        }
    }

    // If there's a tie among candidate LCAs, choose the one with shortest distance to current HEAD
    if let Some(best_lca) = candidate_lcas.iter().min_by_key(|id| {
        (
            dist_current.get(*id).copied().unwrap_or(usize::MAX),
            dist_given.get(*id).copied().unwrap_or(usize::MAX),
        )
    }) {
        Ok(best_lca.clone())
    } else {
        Ok(current_id.to_string())
    }
}

fn collect_ancestors_with_depths(
    repo: &Repo,
    start_id: &str,
) -> Result<(HashSet<String>, HashMap<String, usize>)> {
    let mut ancestors = HashSet::new();
    let mut distances = HashMap::new();
    let mut queue = VecDeque::new();

    ancestors.insert(start_id.to_string());
    distances.insert(start_id.to_string(), 0);
    queue.push_back((start_id.to_string(), 0));

    while let Some((commit_id, dist)) = queue.pop_front() {
        if let Ok(commit) = repo.read_commit(&commit_id) {
            for parent_id in commit.parents {
                if !ancestors.contains(&parent_id) {
                    ancestors.insert(parent_id.clone());
                    distances.insert(parent_id.clone(), dist + 1);
                    queue.push_back((parent_id, dist + 1));
                }
            }
        }
    }

    Ok((ancestors, distances))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Commit;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn make_commit(repo: &Repo, msg: &str, parents: Vec<String>) -> String {
        let commit = Commit::new(
            msg.to_string(),
            "Thu Jan 1 00:00:00 1970 +0000".to_string(),
            parents,
            BTreeMap::new(),
        );
        repo.save_commit(&commit).unwrap()
    }

    #[test]
    fn test_find_split_point_diamond_and_shortcuts() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repo::init(temp_dir.path()).unwrap();
        let init_id = repo.get_head_commit_id().unwrap();

        // Branch A: init -> A1 -> A2
        let a1 = make_commit(&repo, "A1", vec![init_id.clone()]);
        let a2 = make_commit(&repo, "A2", vec![a1.clone()]);

        // Branch B: init -> B1 -> B2
        let b1 = make_commit(&repo, "B1", vec![init_id.clone()]);
        let b2 = make_commit(&repo, "B2", vec![b1.clone()]);

        // Split between A2 and B2 is init
        assert_eq!(find_split_point(&repo, &a2, &b2).unwrap(), init_id);

        // Merge commit M1 on B branch merging A1: parents=[B2, A1]
        let m1 = make_commit(&repo, "M1 (merge A1 into B)", vec![b2.clone(), a1.clone()]);

        // Split point between A2 and M1 must be A1 (not init_id, even though init_id is reachable from both)
        assert_eq!(find_split_point(&repo, &a2, &m1).unwrap(), a1);
    }
}
