use backlogr::{
    cli::{Cli, Command},
    integrations::taiga::{Status, TaigaAPI, UserStories},
    ExitOnError,
};
use clap::Parser;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let username = cli.username.clone();
    let password = cli.password.clone();
    let project_name = cli.project_name.clone();

    let taiga_api = TaigaAPI::authenticate(&username, &password).or_exit();

    let project_id = taiga_api.get_project_id(&project_name).or_exit();

    match cli.command() {
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
                backlogr::cli::Format::Pretty => {
                    let user_stories = UserStories::new(stories);

                    eprintln!("{user_stories}");
                }
                backlogr::cli::Format::Json => {
                    println!("{}", serde_json::to_string_pretty(&stories)?);
                }
            }
        }
    }
    Ok(())
}
