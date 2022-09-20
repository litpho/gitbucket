use std::path::Path;

use git2::build::RepoBuilder;
use tracing::{error, info, trace};

use crate::{
    bitbucket::{all_repositories, BitbucketCredentials},
    errors,
    git::{exclusions::Exclusions, BitbucketRepository, Git},
};

pub async fn run(
    git: Git,
    bitbucket_root_url: &str,
    credentials: &BitbucketCredentials,
    exclusions: Exclusions,
) -> errors::Result<()> {
    let repositories = all_repositories(bitbucket_root_url, credentials, 250)?
        .into_iter()
        .filter(|(project, _)| exclusions.excludes_project(project))
        .flat_map(|(project, repositories)| flatten_repositories(project, repositories))
        .filter(|(project, repository)| exclusions.excludes_repository(project, repository))
        .collect::<Vec<(String, BitbucketRepository)>>();

    std::thread::scope(|s| {
        repositories.into_iter().for_each(|(project, repository)| {
            let project_string = format!("{}/{}", git.root_directory, project);
            s.spawn(|| {
                let span = tracing::info_span!(
                    "clone_repository",
                    "{}/{}",
                    project_string,
                    repository.name
                );
                if let Err(e) = span.in_scope(|| clone_repository(&git, project_string, repository))
                {
                    error!("Error {:?}", e);
                }
            });
        });
    });

    Ok(())
}

fn flatten_repositories(
    project: String,
    repositories: Vec<BitbucketRepository>,
) -> impl Iterator<Item = (String, BitbucketRepository)> {
    repositories.into_iter().map(move |r| (project.clone(), r))
}

fn clone_repository(
    git: &Git,
    project: String,
    repository: BitbucketRepository,
) -> errors::Result<()> {
    let repo_string = format!("{}/{}", project, repository.name);
    let repo_path = Path::new(&repo_string);
    trace!("Checking repository");
    if !repo_path.exists() {
        info!("Cloning repository from {}", &repository.git_url);
        if !git.dry_run {
            do_clone(git, repo_path, &repository.git_url)?;
        }
    }

    Ok(())
}

fn do_clone(git: &Git, repo_path: &Path, git_url: &str) -> errors::Result<()> {
    let fo = git.fetch_options();

    // Prepare builder.
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    builder.clone(git_url, repo_path)?;

    Ok(())
}
