use crate::bitbucket::BitbucketRepository;

#[derive(Debug)]
pub struct Exclusions {
    exclusions: Vec<String>,
}

impl From<Option<String>> for Exclusions {
    fn from(excluded_projects: Option<String>) -> Self {
        let exclusions = excluded_projects.map_or_else(Vec::new, |val| {
            val.split(',')
                .map(Self::add_wildcard)
                .collect::<Vec<String>>()
        });

        Exclusions { exclusions }
    }
}

impl Exclusions {
    pub fn excludes_project(&self, project: &str) -> bool {
        !self.exclusions.contains(&format!("{}/*", project))
    }

    pub fn excludes_repository(&self, project: &str, repository: &BitbucketRepository) -> bool {
        !self
            .exclusions
            .contains(&format!("{}/{}", project, repository.name))
    }

    fn add_wildcard(s: &str) -> String {
        match s.contains('/') {
            true => s.trim().to_owned(),
            false => format!("{}/*", s.trim()),
        }
    }
}
