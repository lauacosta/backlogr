use clap::{Parser, Subcommand, ValueEnum, command, crate_version};

use crate::integrations::taiga::Status;

#[derive(Parser)]
#[command(version, about,  long_about = None, before_help = format!(r#"
▗▄▄▖ ▗▞▀▜▌▗▞▀▘█  ▄ █  ▄▄▄   ▄▄▄ 
▐▌ ▐▌▝▚▄▟▌▝▚▄▖█▄▀  █ █   █ █    
▐▛▀▚▖         █ ▀▄ █ ▀▄▄▄▀ █    
▐▙▄▞▘         █  █ █     ▗▄▖    
                        ▐▌ ▐▌   
                         ▝▀▜▌   
                        ▐▙▄▞▘   
                            
    @lauacosta/backlogr {}"#, crate_version!()
    ))
]
pub struct Cli {
    /// Taiga Username
    #[arg(long = "username", env = "USERNAME", required = true)]
    pub username: String,

    /// Taiga password
    #[arg(long = "password", env = "PASSWORD", required = true)]
    pub password: String,

    /// Taiga project name
    #[arg(long = "project_name", env = "PROJECT_NAME", required = true)]
    pub project_name: String,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    #[must_use]
    pub fn command(&self) -> Command {
        self.command.clone().unwrap_or(Command::List {
            format: Format::Pretty,
        })
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Creates a new User Story
    Create {
        #[arg(long = "subject")]
        subject: String,
        #[arg(long = "description")]
        description: Option<String>,
        #[arg(long = "description", value_enum, default_value_t = Status::New)]
        status: Status,
    },
    /// Updates a User Story to 'In Progress'
    Wip { story_id: usize },
    /// Updates a User Story to 'Done'
    Done { story_id: usize },
    /// Deletes a User Story
    Delete { story_id: usize },
    /// List User stories
    List {
        #[arg(short, long = "format", value_enum, default_value_t = Format::Pretty)]
        format: Format,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Pretty,
    Json,
}
