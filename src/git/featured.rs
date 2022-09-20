use git2::{ErrorCode, Reference, Repository};
use std::path::Path;
use tracing::{error, info, trace, Instrument};

use crate::{errors, git::Git};

pub async fn run(git: Git, show_main: bool) -> errors::Result<()> {
    for project in &git.get_existing_git_projects()? {
        featured(project, show_main)
            .instrument(tracing::info_span!(
                "featured",
                "{}",
                project.to_string_lossy()
            ))
            .await?;
    }

    Ok(())
}

async fn featured(directory: &Path, show_main: bool) -> errors::Result<()> {
    let repo = Repository::open(directory)?;
    match repo.head() {
        Ok(head) => debug_head(directory, &head, show_main).await?,
        Err(e) => {
            if e.code() != ErrorCode::UnbornBranch {
                error!("Error in path: - {}", e);
            }
        }
    };

    Ok(())
}

async fn debug_head(directory: &Path, head: &Reference<'_>, show_main: bool) -> errors::Result<()> {
    trace!("Checking directory {}", directory.to_string_lossy());

    if head.is_branch() {
        match head.shorthand() {
            Some(branchname) => match branchname {
                "main" | "master" | "develop" => {
                    if show_main {
                        info!("on branch {}", branchname);
                    }
                }
                _ => {
                    info!("on branch {}", branchname);
                }
            },
            None => {
                error!("not on a branch");
            }
        }
    } else {
        error!("not a branch");
    }

    Ok(())
}
