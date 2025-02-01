use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};

/// Simple CLI demonstration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Optional name to greet
    #[arg(short, long)]
    pub name: Option<String>,
}

/// Get the user's name either from command line args or through interactive prompt
pub fn get_user_name(args_name: Option<String>) -> String {
    match args_name {
        Some(name) => name,
        None => Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("What's your name?")
            .interact_text()
            .unwrap(),
    }
}
