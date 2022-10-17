mod input;
mod db;
mod report;

use std::{process::Command, process::ExitCode, path::Path};
use db::{Database, BackupableDatabase, jsondb::JsonDb};
use clap::{Parser, Subcommand};
use dialoguer::console::style;
use report::{Report, client_report};

#[derive(Parser)]
#[command(about="this program will always remain manjaliof")]
struct Cli {
    #[command(subcommand)] 
    command: Commands
}

#[derive(Subcommand, PartialEq)]
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

const DB_FILE_PATH: &str = "manjaliof-data/data.json";
const POST_SCRIPTS_FOLDER_PATH: &str = "manjaliof-data/post_scripts";
const MAP_COMMANDS_WITH_POST_SCRIPT: [(Commands, &str); 3] = [
    (Commands::Add, "postadd"),
    (Commands::Renew, "postrenew"),
    (Commands::Delete, "postdelete")
];

fn main() -> ExitCode {
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            let error_msg = format!("Error: {}", err);
            eprintln!("{}", style(error_msg).red());
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), String> {
    let args = Cli::parse();

    let db = JsonDb::new(DB_FILE_PATH);
    let backup = db.get_backup()?;

    let command_function = match args.command {
        Commands::Add => add_client,
        Commands::Renew => renew_client,
        Commands::Delete => delete_client,
        Commands::List => list_clients,
        Commands::HealthCheck => health_check
    };

    let result = command_function(&db, get_command_post_script(&args.command));
    if result.is_err() && args.command != Commands::List {
        db.restore_backup(backup)?;
    }

    result
}

fn add_client(db: &dyn Database, post_script_name: Option<&str>) -> Result<(), String> {
    let name = input::get_client_name();
    let months = input::get_months();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    db.add_client(&name, months, &seller, money)?;

    run_post_script(post_script_name.unwrap(), &name)?;
    println!("{}", style("client added successfully").green());
    Ok(())
}

fn renew_client(db: &dyn Database, post_script_name: Option<&str>) -> Result<(), String> {
    let name = input::get_client_name();
    let months = input::get_months();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    db.renew_client(&name, months, &seller, money)?;

    run_post_script(post_script_name.unwrap(), &name)?;
    println!("{}", style("client renewed successfully").green());
    Ok(())
}

fn delete_client(db: &dyn Database, post_script_name: Option<&str>) -> Result<(), String> {
    let name = input::get_client_name();
    db.delete_client(&name)?;

    run_post_script(post_script_name.unwrap(), &name)?;
    println!("{}", style("client deleted successfully").green());
    Ok(())
}

fn list_clients(db: &dyn Database, _post_script_name: Option<&str>) -> Result<(), String> {
    let mut clients = db.list_clients()?;
    clients.sort_by_key(|client| client.expire_time);
    clients.reverse();

    let mut report = Report::new(["name", "months left", "seller"].to_vec());
    for client in clients {
        let name = style(client.name).cyan().to_string();
        let months_left = client_report::calculate_months_left(client.expire_time);
        let sellers = client_report::calculate_sellers(&client.payments);

        report.add_item([name, months_left, sellers].to_vec());
    }

    report.show();
    Ok(())
}

fn health_check(db: &dyn Database, _post_script_name: Option<&str>) -> Result<(), String> {
    let db_file = Path::new(DB_FILE_PATH);
    if !db_file.is_file() {
        return Err(format!("cannot find database file at '{}'", DB_FILE_PATH));
    }

    if db.list_clients().is_err() {
        return Err("cannot extract data from db".to_string());
    }

    let post_scripts_folder = Path::new(POST_SCRIPTS_FOLDER_PATH);
    if !post_scripts_folder.is_dir() {
        return Err(format!("cannot find post scripts folder at '{}'", POST_SCRIPTS_FOLDER_PATH));
    }

    for command_with_post_script in MAP_COMMANDS_WITH_POST_SCRIPT {
        let post_script_file_name = command_with_post_script.1;
        let post_script = post_scripts_folder.join(post_script_file_name);
        if !post_script.is_file() {
            return Err(format!("cannot find post script at '{}'", post_script.to_str().unwrap()));
        }
    }

    let success_msg = format!("everything is fine");
    println!("{}", style(success_msg).green());
    Ok(())
}

fn get_command_post_script(command: &Commands) -> Option<&'static str> {
    match MAP_COMMANDS_WITH_POST_SCRIPT.iter().find(|command_with_post_script| command_with_post_script.0 == *command) {
        Some(command_with_post_script) => Some(command_with_post_script.1),
        None => None
    }
}

fn run_post_script(script_name: &str, arg: &str) -> Result<(), String> {
    let script_path = Path::new(POST_SCRIPTS_FOLDER_PATH).join(script_name);
    let output = Command::new(&script_path).arg(arg).output().map_err(
        |error| format!("couldn't run post script '{}': {}", script_path.to_str().unwrap(), error.to_string()))?;

    if !output.status.success() {
        let mut output_stderr = std::str::from_utf8(output.stderr.as_slice()).unwrap().to_string();
        output_stderr.pop(); // remove new line
        return Err(format!("post script exited due to a failure: {}", output_stderr));
    }

    let result = std::str::from_utf8(output.stdout.as_slice()).unwrap().to_string();
    if !result.is_empty() {
        println!("{}", result);
    }
    Ok(())
}
