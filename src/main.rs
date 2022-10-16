mod input;
mod db;

use std::{process::ExitCode, path::Path};
use db::{Database, jsondb::JsonDb};
use clap::{Parser, Subcommand};
use dialoguer::console::style;

#[derive(Parser)]
#[command(about="this program will always remain manjaliof")]
struct Cli {
    #[command(subcommand)] 
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    #[command(about="adds new client to db")]
    Add,

    #[command(about="renew client")]
    Renew,

    #[command(about="delete client")]
    Delete,

    #[command(about="show all clients")]
    List,

    #[command(about="check db and post scripts exist")]
    HealthCheck
}

const DB_FILE_PATH: &'static str = "manjaliof-data/data.json";
const POST_SCRIPTS_FOLDER_PATH: &'static str = "manjaliof-data/post_scripts";
const POST_SCRIPTS_FILE_NAMES: [&'static str; 3] = ["postadd", "postrenew", "postdelete"];

fn main() -> ExitCode {
    let args = Cli::parse();
    let db = JsonDb::new(DB_FILE_PATH);

    let command_function = match args.command {
        Commands::Add => add_client,
        Commands::Renew => renew_client,
        Commands::Delete => delete_client,
        Commands::List => todo!(),
        Commands::HealthCheck => health_check
    };

    if let Err(error) = command_function(&db) {
        let complete_error_msg = format!("Error: {}", error);
        eprintln!("{}", style(complete_error_msg).red());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn add_client(db: &dyn Database) -> Result<(), String> {
    let name = input::get_client_name();
    let months = input::get_months();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    db.add_client(name, months, seller, money)
}

fn renew_client(db: &dyn Database) -> Result<(), String> {
    let name = input::get_client_name();
    let months = input::get_months();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    db.renew_client(name, months, seller, money)
}

fn delete_client(db: &dyn Database) -> Result<(), String> {
    let name = input::get_client_name();
    db.delete_client(name)
}

fn health_check(db: &dyn Database) -> Result<(), String> {
    let db_file = Path::new(DB_FILE_PATH);
    if !db_file.is_file() {
        return Err(format!("Cannot find database file at '{}'", DB_FILE_PATH));
    }

    if db.list_clients().is_err() {
        return Err("Cannot extract data from db".to_string());
    }

    let post_scripts_folder = Path::new(POST_SCRIPTS_FOLDER_PATH);
    if !post_scripts_folder.is_dir() {
        return Err(format!("Cannot find post scripts folder at '{}'", POST_SCRIPTS_FOLDER_PATH));
    }

    for post_script_file_name in POST_SCRIPTS_FILE_NAMES {
        let post_script = post_scripts_folder.join(post_script_file_name);
        if !post_script.is_file() {
            return Err(format!("Cannot find post script at '{}'", post_script.to_str().unwrap()));
        }
    }

    let success_msg = format!("Everything is fine");
    println!("{}", style(success_msg).green());
    Ok(())
}
