use git2::{build::CheckoutBuilder, ErrorCode, Repository};
use std::path::Path;
use tracing::{debug, error, info, trace};

use crate::{errors, git::Git};

pub async fn run(git: Git, show_errors: bool) -> errors::Result<()> {
    let existing_projects = git.get_existing_git_projects()?;
    std::thread::scope(|s| {
        existing_projects.iter().for_each(|project| {
            let git = git.clone();
            s.spawn(move || {
                let span = tracing::info_span!("pull", "{}", project.to_string_lossy());
                if let Err(e) = span.in_scope(|| fast_forward(&git, project, show_errors)) {
                    error!("Error {:?}", e);
                }
            });
        })
    });

    Ok(())
}

fn fast_forward(git: &Git, directory: &Path, show_errors: bool) -> errors::Result<()> {
    trace!("Checking repo");
    let repo = Repository::open(&directory)?;
    if !Git::is_clean(&repo)? {
        if show_errors {
            debug!("Repository not clean");
        }
        return Ok(());
    }

    let branch = match branch(&repo) {
        Ok(branch) => branch,
        Err(e) => {
            if let errors::Error::FailedGitOperation(e) = e {
                if e.code() != ErrorCode::UnbornBranch {
                    error!("Branch not found - {}", e);
                }
            }
            return Ok(());
        }
    };

    if check_path(git, &repo, &branch).is_err() {
        return Ok(());
    }

    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(head) => head,
        Err(e) => {
            error!("Error fetching head - {}", e);
            return Ok(());
        }
    };

    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.is_up_to_date() {
        trace!("up to date");
        Ok(())
    } else if analysis.is_fast_forward() {
        info!("fast-forwarding");
        if !git.dry_run {
            let refname = format!("refs/heads/{}", &branch);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force()))
                .map_err(errors::Error::FailedGitOperation)?;
        }
        Ok(())
    } else {
        debug!("Can't fast-forward");
        Ok(())
    }
}

const MAX_RETRIES: usize = 5;

fn check_path(git: &Git, repo: &Repository, branch: &str) -> errors::Result<()> {
    for _ in 1..MAX_RETRIES {
        let result =
            repo.find_remote("origin")?
                .fetch(&[branch], Some(&mut git.fetch_options()), None);
        match result {
            Ok(_) => return Ok(()),
            Err(e) => {
                if e.code() != ErrorCode::UnbornBranch
                    && !e.message().contains("failed to start SSH session")
                {
                    error!("Error fetching path - {}", e);
                    return Err(errors::Error::FailedGitOperation(e));
                }
            }
        }
    }

    Ok(())
}

fn branch(repository: &Repository) -> errors::Result<String> {
    let head = repository.head()?;
    match head.is_branch() {
        true => head
            .shorthand()
            .ok_or(errors::Error::NoBranchnameFound)
            .map(|x| x.to_owned()),
        false => Err(errors::Error::NoBranchFound),
    }
}
