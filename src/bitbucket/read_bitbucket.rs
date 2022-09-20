//! Private implementation for querying the Bitbucket REST API

use serde::Deserialize;
use std::collections::HashMap;
use ureq::Agent;

use crate::{
    bitbucket::{BitbucketCredentials, BitbucketRepository},
    errors,
};

/// Query all repositories per project from the Bitbucket REST API
pub fn get_all_repositories(
    bitbucket_root_url: &str,
    credentials: &BitbucketCredentials,
    limit: i16,
) -> errors::Result<HashMap<String, Vec<BitbucketRepository>>> {
    let agent: Agent = ureq::AgentBuilder::new().build();
    let authorization = String::from(credentials);

    let mut repos: HashMap<String, Vec<BitbucketRepository>> = HashMap::new();
    let mut start: i16 = 0;
    loop {
        let response = agent
            .get(format!("{}/rest/api/latest/repos", bitbucket_root_url).as_str())
            .query("start", &i16::to_string(&start))
            .query("limit", &i16::to_string(&limit))
            .set("Authorization", &authorization)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(401, _) => errors::Error::InvalidUsernamePassword,
                _ => errors::Error::FailedToDoHttpCall(Box::new(e)),
            })?;

        let json: RemoteEnvelope = response
            .into_json()
            .map_err(errors::Error::FailedToParseJSON)?;

        for value in &json.values {
            if value.project.project_type == "NORMAL" {
                let project_key = &value.project.key;
                let data = BitbucketRepository::builder()
                    .name(&value.name)
                    .git_url(value.ssh_url().ok_or(errors::Error::SshUrlMissing)?)
                    .build();
                repos
                    .entry(project_key.to_owned())
                    .or_insert_with(Vec::new)
                    .push(data);
            }
        }
        if json.is_last_page {
            break;
        }
        start = json.next_page_start.unwrap()
    }

    Ok(repos)
}

/// The outer envelope from the JSON reply containing pagination and data (values)
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RemoteEnvelope {
    is_last_page: bool,
    next_page_start: Option<i16>,
    values: Vec<RemoteRepository>,
}

/// A Remote Repository representation containing all remote links
#[derive(Deserialize, Debug)]
struct RemoteRepository {
    name: String,
    project: RemoteProject,
    links: HashMap<String, Vec<RemoteLink>>,
}

impl RemoteRepository {
    /// Get the ssh_url from the "clone" link
    fn ssh_url(&self) -> Option<&str> {
        let clone = &self.links.get("clone");
        if let Some(clone) = clone {
            if let Some(link) = clone
                .iter()
                .find(|link| link.name == Some("ssh".to_owned()))
            {
                return Some(&link.href);
            }
        }
        None
    }
}

#[derive(Deserialize, Debug)]
struct RemoteProject {
    key: String,
    #[serde(alias = "type")]
    project_type: String,
}

#[derive(Deserialize, Debug)]
struct RemoteLink {
    href: String,
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_without_repositories() {
        let json = r#"
        {
            "isLastPage": true,
            "size": 1,
            "limit": 2,
            "values": []
        }
        "#;
        let result: RemoteEnvelope = serde_json::from_str(json).unwrap();

        assert_eq!(result.is_last_page, true);
        assert_eq!(result.size, 1);
        assert_eq!(result.limit, 2);
        assert!(result.next_page_start.is_none());
        assert!(result.values.is_empty());
    }

    #[test]
    fn deserialize_with_repositories() {
        let json = r#"
        {
            "isLastPage": true,
            "size": 1,
            "limit": 2,
            "values": [
                {
                    "name": "ATLAS",
                    "project": {
                        "key": "ATLAS",
                        "type": "NORMAL"
                    },
                    "links": {
                        "dummy": [
                            {
                                "href": "href",
                                "name": "name"
                            }
                        ],
                        "clone": [
                            {
                                "href": "href",
                                "name": "name"
                            }
                        ]
                    }
                }
            ]
        }
        "#;
        let result: RemoteEnvelope = serde_json::from_str(json).unwrap();

        assert_eq!(result.is_last_page, true);
        assert_eq!(result.size, 1);
        assert_eq!(result.limit, 2);
        assert!(result.next_page_start.is_none());
        assert_eq!(result.values.len(), 1);

        let result: &RemoteRepository = result.values.get(0).unwrap();

        assert_eq!(result.name, "ATLAS");
        assert_eq!(result.project.key, "ATLAS");
        assert_eq!(result.project.project_type, "NORMAL");
        assert_eq!(result.links.get("clone").unwrap().len(), 1);

        let result: &RemoteLink = result.links.get("clone").unwrap().get(0).unwrap();

        assert_eq!(result.href, "href");
        assert_eq!(result.name, Some("name".to_string()));
    }

    #[test]
    fn ssh_url() {
        assert_eq!(
            remote_repository("clone", "href", Some("ssh")).ssh_url(),
            Some("href")
        );
        assert_eq!(
            remote_repository("kloon", "href", Some("ssh")).ssh_url(),
            None
        );
        assert_eq!(remote_repository("clone", "href", None).ssh_url(), None);
        assert_eq!(
            remote_repository("clone", "href", Some("xxx")).ssh_url(),
            None
        );
    }

    fn remote_repository(link_name: &str, href: &str, href_name: Option<&str>) -> RemoteRepository {
        let name = String::from("name");
        let project = RemoteProject {
            key: String::from("ATLAS"),
            project_type: String::from("NORMAL"),
        };
        let mut links: HashMap<String, Vec<RemoteLink>> = HashMap::new();
        links.insert(
            link_name.to_string(),
            vec![RemoteLink {
                href: href.to_string(),
                name: href_name.map(|x| x.to_string()),
            }],
        );
        links.insert(
            String::from("extra"),
            vec![RemoteLink {
                href: String::from("fehr"),
                name: None,
            }],
        );

        RemoteRepository {
            name,
            project,
            links,
        }
    }
}
