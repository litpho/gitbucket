use git2::Repository;
use std::path::Path;
use tracing::{error, info, trace};

use crate::{errors, git::Git};

pub async fn run(git: Git) -> errors::Result<()> {
    let existing_projects = git.get_existing_git_projects()?;
    std::thread::scope(|s| {
        existing_projects.iter().for_each(|directory| {
            s.spawn(move || {
                let span = tracing::info_span!("status", "{}", directory.to_string_lossy());
                if let Err(e) = span.in_scope(|| status(directory)) {
                    error!("Error {:?}", e);
                }
            });
        });
    });

    Ok(())
}

fn status(directory: &Path) -> errors::Result<()> {
    trace!("Checking directory");

    let repo = Repository::open(directory)?;
    if !Git::is_clean(&repo)? {
        info!("Directory is dirty");
    }

    Ok(())
}
