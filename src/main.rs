mod input;
mod db;

use std::process::ExitCode;
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
    List
}

fn main() -> ExitCode {
    let args = Cli::parse();
    let db = JsonDb::new("data.json");

    let command_function = match args.command {
        Commands::Add => add_client,
        Commands::Renew => renew_client,
        Commands::Delete => delete_client,
        Commands::List => todo!()
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
