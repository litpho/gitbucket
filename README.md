# Gitbucket
## Environment variables 

- GITBUCKET_DIRECTORY
: Root directory for repositories
- GITBUCKET_EXCLUDED_PROJECTS
: Projects/repositories excluded from download
- GITBUCKET_PRIVATE_KEY
: private SSL key, default ~/.ssh/id.rsa
- GITBUCKET_ROOT_URL
: root url for Bitbucket
- GITBUCKET_USER
: The user used for Bitbucket access

## Command's

### General
Gitbucket can be called as `gitbucket --help` or `gitbucket <command> --help` to show the specific parameters

### Clone
Clone all repositories that do not exist locally
`gitbucket clone`
### Featured
Show all repositories currently on a branch other than main/master/develop
### Pull
Pull all repositories that don't have changes in their workspace and are on branches main/master/develop
### Status
Show all repositories with changes in their workspace
