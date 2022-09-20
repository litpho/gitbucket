//! This module contains everything to do with parsing commandline arguments and transforming them into Commands.

use clap::{Args, Parser, Subcommand};
use gitbucket::{
    bitbucket::BitbucketCredentials,
    errors,
    git::{exclusions::Exclusions, Git},
};
use std::env;

pub enum SubCommand {
    Clone {
        git: Git,
        bitbucket_root_url: String,
        credentials: BitbucketCredentials,
        exclusions: Exclusions,
    },
    Featured {
        git: Git,
        show_main: bool,
    },
    Pull {
        git: Git,
        show_errors: bool,
    },
    Status {
        git: Git,
    },
}

impl SubCommand {
    pub fn from_arguments() -> errors::Result<SubCommand> {
        let cli: Cli = Cli::parse();

        let git = Git::builder()
            .root_directory(cli.args.directory)
            .private_key_location(cli.args.private_key)
            .dry_run(cli.args.dry_run)
            .build();

        let command = match cli.command {
            CliCommands::Clone {
                user,
                password,
                bitbucket_root_url,
            } => {
                let password = password
                    .unwrap_or_else(|| rpassword::prompt_password("Bitbucket password: ").unwrap());
                let credentials = BitbucketCredentials::builder()
                    .username(user)
                    .password(password)
                    .build();
                let exclusions = Exclusions::from(cli.args.excluded_projects);
                SubCommand::Clone {
                    git,
                    bitbucket_root_url,
                    credentials,
                    exclusions,
                }
            }
            CliCommands::Featured { show_main } => SubCommand::Featured { git, show_main },
            CliCommands::Pull { show_errors } => SubCommand::Pull { git, show_errors },
            CliCommands::Status => SubCommand::Status { git },
        };

        Ok(command)
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    args: CliArgs,
    #[clap(subcommand)]
    command: CliCommands,
}

#[derive(Debug, Args)]
struct CliArgs {
    #[clap(
        short,
        long,
        name = "DIR",
        help = "Sets the root directory",
        env = "GITBUCKET_DIRECTORY",
        default_value = "."
    )]
    directory: String,
    #[clap(
    long,
    name = "KEYFILE",
    help = "Sets the private key location",
    env = "GITBUCKET_PRIVATE_KEY",
    default_value_t = CliArgs::default_private_key().unwrap()
    )]
    private_key: String,
    #[clap(long, help = "Runs a dry run")]
    dry_run: bool,
    #[clap(
        long,
        help = "Excluded projects",
        required = false,
        env = "GITBUCKET_EXCLUDED_PROJECTS"
    )]
    excluded_projects: Option<String>,
}

impl CliArgs {
    fn default_private_key() -> errors::Result<String> {
        let home_dir_location =
            env::var("HOME").map_err(errors::Error::HOMEEnvironmentVariableNotFound)?;
        Ok(format!("{}/.ssh/id_rsa", home_dir_location))
    }
}

#[derive(Debug, Subcommand)]
enum CliCommands {
    #[clap(about = "clone new repositories")]
    Clone {
        #[clap(
            short,
            long,
            help = "Bitbucket user",
            required = true,
            env = "GITBUCKET_USER"
        )]
        user: String,
        #[clap(long, help = "Bitbucket password", required = false)]
        password: Option<String>,
        #[clap(
            long,
            help = "Bitbucket root url",
            required = false,
            env = "GITBUCKET_ROOT_URL"
        )]
        bitbucket_root_url: String,
    },
    #[clap(about = "show repositories not on main/master/develop")]
    Featured {
        #[clap(long, help = "show main/master/develop branches")]
        show_main: bool,
    },
    #[clap(about = "pull and update clean repositories")]
    Pull {
        #[clap(long, help = "show errors")]
        show_errors: bool,
    },
    #[clap(about = "status from repositories")]
    Status,
}
