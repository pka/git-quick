use chrono::{TimeZone, Utc};
use git2;

/// Returns the age of the last commit in hours.
pub fn num_hours_since_last_commit(git2_repo: &git2::Repository) -> i64 {
    // dbg!(git2_repo.state());
    if let Ok(head) = git2_repo.head() {
        if let Some(oid) = head.target() {
            if let Ok(commit) = git2_repo.find_commit(oid) {
                let commit_time = Utc.timestamp(commit.time().seconds(), 0);
                let age_h = Utc::now().signed_duration_since(commit_time).num_hours();
                return age_h;
            }
        }
    }
    i64::max_value()
}
