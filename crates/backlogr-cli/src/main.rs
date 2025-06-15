use backlogr_taiga::{ExitOnError, Status, TaigaAPI, UserStories};
use clap::{Parser, Subcommand, ValueEnum, crate_version};
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use facet_pretty::FacetPretty;

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
pub struct Args {
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

impl Args {
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

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let username = args.username.clone();
    let password = args.password.clone();
    let project_name = args.project_name.clone();

    let taiga_api = TaigaAPI::authenticate(&username, &password).or_exit();

    let project_id = taiga_api.get_project_id(&project_name).or_exit();

    match args.command() {
        Command::Create {
            subject,
            description,
            status,
        } => {
            let description = description.unwrap_or_default();
            let story_id = taiga_api
                .create_story(project_id, &subject, &description, &status)
                .or_exit();

            eprintln!(
                "✅ Created story: \"{subject}\" (#{})",
                story_id.bold().bright_green()
            );
        }
        Command::Wip { story_id } => {
            let real_id = taiga_api.get_story_id(project_id, story_id).or_exit();

            taiga_api
                .update_story_status(project_id, story_id, real_id, &Status::Wip)
                .or_exit();
        }
        Command::Done { story_id } => {
            let real_id = taiga_api.get_story_id(project_id, story_id).or_exit();

            taiga_api
                .update_story_status(project_id, story_id, real_id, &Status::Done)
                .or_exit();
        }
        Command::Delete { story_id } => {
            let real_id = taiga_api.get_story_id(project_id, story_id).or_exit();

            taiga_api.delete_story(real_id).or_exit();

            eprintln!(
                "✅ Successfully deleted user story (#{})",
                story_id.bold().bright_green(),
            );
        }
        Command::List { format } => {
            let stories = taiga_api.list_all_stories(project_id).or_exit();

            match format {
                Format::Pretty => {
                    let user_stories = UserStories::new(stories);

                    eprintln!("{user_stories}");
                }
                Format::Json => {
                    println!("{}", facet_json::to_string(&stories).pretty());
                }
            }
        }
    }
    Ok(())
}
