//! This module contains the logic for implementing the Git commands

pub mod exclusions;

mod clone;
mod featured;
mod pull;
mod status;

use git2::{Cred, FetchOptions, RemoteCallbacks, Repository, StatusOptions};
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use typed_builder::TypedBuilder;

use crate::{
    bitbucket::{BitbucketCredentials, BitbucketRepository},
    errors,
    git::exclusions::Exclusions,
};

#[derive(TypedBuilder, Clone, Debug)]
pub struct Git {
    #[builder(setter(into))]
    root_directory: String,
    #[builder(setter(into))]
    private_key_location: PathBuf,
    dry_run: bool,
}

impl Git {
    pub async fn clone_command(
        self,
        bitbucket_root_url: &str,
        credentials: &BitbucketCredentials,
        excluded_projects: Exclusions,
    ) -> errors::Result<()> {
        clone::run(self, bitbucket_root_url, credentials, excluded_projects).await
    }

    pub async fn featured_command(self, show_main: bool) -> errors::Result<()> {
        featured::run(self, show_main).await
    }

    pub async fn pull_command(self, show_errors: bool) -> errors::Result<()> {
        pull::run(self, show_errors).await
    }

    pub async fn status_command(self) -> errors::Result<()> {
        status::run(self).await
    }

    fn fetch_options(&self) -> FetchOptions {
        // Prepare callbacks.
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                &self.private_key_location,
                None,
            )
        });

        // Prepare fetch options.
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);
        fo
    }

    fn get_existing_git_projects(&self) -> errors::Result<Vec<PathBuf>> {
        let paths = fs::read_dir(&self.root_directory)
            .map_err(|source| errors::Error::FailedToReadDirectory {
                directory: (&self.root_directory).clone(),
                source,
            })?
            .filter(|dir_entry| self.is_dir_and_not_symlink(dir_entry.as_ref().unwrap()))
            .map(|dir_entry| dir_entry.unwrap().path())
            .flat_map(|path| self.get_existing_git_repos(&path).unwrap())
            .collect::<Vec<PathBuf>>();

        Ok(paths)
    }

    fn is_clean(repository: &Repository) -> errors::Result<bool> {
        let statuses = repository.statuses(Some(StatusOptions::new()).as_mut())?;
        Ok(statuses.is_empty())
    }

    fn get_existing_git_repos(&self, dir: &Path) -> errors::Result<Vec<PathBuf>> {
        let paths = fs::read_dir(dir)
            .map_err(|source| errors::Error::FailedToReadDirectory {
                directory: (&self.root_directory).clone(),
                source,
            })?
            .filter(|dir_entry| self.is_dir_and_not_symlink(dir_entry.as_ref().unwrap()))
            .map(|dir_entry| dir_entry.unwrap().path())
            .filter(|path| path.join(".git").exists())
            .collect::<Vec<PathBuf>>();

        Ok(paths)
    }

    fn is_dir_and_not_symlink(&self, dir_entry: &DirEntry) -> bool {
        let file_type = dir_entry.file_type().unwrap();
        file_type.is_dir() && !file_type.is_symlink()
    }
}
