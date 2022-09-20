//! This module contains methods for connecting to the Bitbucket Server REST API.

mod read_bitbucket;

use std::collections::HashMap;
use typed_builder::TypedBuilder;

use crate::errors;

/// A BitbucketRepository result type
#[derive(TypedBuilder, Clone, Debug)]
pub struct BitbucketRepository {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub git_url: String,
}

#[derive(TypedBuilder)]
pub struct BitbucketCredentials {
    #[builder(setter(into))]
    username: String,
    #[builder(setter(into))]
    password: String,
}

/// Query all repositories per project from the Bitbucket REST API
pub fn all_repositories(
    bitbucket_root_url: &str,
    credentials: &BitbucketCredentials,
    limit: i16,
) -> errors::Result<HashMap<String, Vec<BitbucketRepository>>> {
    read_bitbucket::get_all_repositories(bitbucket_root_url, credentials, limit)
}

impl From<&BitbucketCredentials> for String {
    fn from(credentials: &BitbucketCredentials) -> Self {
        format!(
            "Basic {}",
            base64::encode(format!("{}:{}", credentials.username, credentials.password))
        )
    }
}
