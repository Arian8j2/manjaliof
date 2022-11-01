mod input;
mod db;
mod report;

use std::{env, process::Command, process::ExitCode, path::Path};
use db::{Database, BackupableDatabase, jsondb::JsonDb, Target};
use clap::{Args, Parser, Subcommand};
use dialoguer::console::style;
use report::{Report, client_report};

#[derive(Parser)]
#[command(about="this program will always remain manjaliof")]
struct Cli {
    #[command(subcommand)] 
    command: Commands,

    #[arg(long, default_value_t = false)]
    skip_post_script: bool
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

    #[command(about="set client info")]
    SetInfo(SetInfoArgs)
}

#[derive(Args, PartialEq)]
struct SetInfoArgs {
    #[arg(short, long, default_value_t = false)]
    all: bool
}

const DATA_PATH_ENV_NAME: &str = "MANJALIOF_DATA";
const DB_FILE_NAME: &str = "data.json";
const POST_SCRIPTS_FOLDER_NAME: &str = "post_scripts";
const MAP_COMMANDS_WITH_POST_SCRIPT: [(Commands, &str); 3] = [
    (Commands::Add, "add"),
    (Commands::Renew, "renew"),
    (Commands::Delete, "delete")
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

    let db_path = Path::new(&get_data_path()?).join(DB_FILE_NAME);
    let db = JsonDb::new(db_path.to_str().unwrap());
    let backup = db.get_backup()?;

    let post_script_name = get_command_post_script(&args.command, args.skip_post_script);
    let result = {
        let post_script_arg = match &args.command {
            Commands::Add => add_client(&db)?,
            Commands::Renew => renew_client(&db)?,
            Commands::Delete => delete_client(&db)?,
            Commands::List => list_clients(&db)?,
            Commands::SetInfo(args) => set_client_info(&db, args)?,
        };

        if let (Some(name), Some(arg)) = (post_script_name, post_script_arg){
            run_post_script(name, &arg)?;
        }
        
        println!("{}", style("command executed successfully").green());
        Ok(())
    };

    if result.is_err() && args.command != Commands::List {
        db.restore_backup(backup)?;
    }

    result
}

fn add_client(db: &dyn Database) -> Result<Option<String>, String> {
    let name = input::get_client_name();
    let days = input::get_days();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    let info = input::get_info(None);
    db.add_client(&name, days, &seller, money, &info)?;

    Ok(Some(name))
}

fn renew_client(db: &dyn Database) -> Result<Option<String>, String> {
    let name = input::get_client_name();
    let days = input::get_days();
    let seller = input::get_seller();
    let money = input::get_money_amount();
    db.renew_client(&name, days, &seller, money)?;

    Ok(Some(name))
}

fn delete_client(db: &dyn Database) -> Result<Option<String>, String> {
    let name = input::get_client_name();
    db.delete_client(&name)?;

    Ok(Some(name))
}

fn list_clients(db: &dyn Database) -> Result<Option<String>, String> {
    let mut clients = db.list_clients()?;
    clients.sort_by_key(|client| client.expire_time);
    clients.reverse();

    let mut report = Report::new(["name", "months left", "seller", "info"].to_vec());
    for client in clients {
        let name = style(client.name).cyan().to_string();
        let months_left = client_report::calculate_days_left(client.expire_time);
        let sellers = client_report::calculate_sellers(&client.payments);
        let info = style(client.info.unwrap_or("".to_string())).black().bright().to_string();

        report.add_item([name, months_left, sellers, info].to_vec());
    }

    report.show();
    Ok(None)
}

fn set_client_info(db: &dyn Database, args: &SetInfoArgs) -> Result<Option<String>, String> {
    let target: Target = if args.all { Target::All } else { Target::OnePerson(input::get_client_name()) };
    let last_info = match &target {
        Target::OnePerson(name) => db.get_client_info(&name)?,
        Target::All => "".to_string()
    };

    let new_info = input::get_info(Some(&last_info));
    db.set_client_info(target, &new_info)?;
    Ok(None)
}

fn get_command_post_script(command: &Commands, skip: bool) -> Option<&'static str> {
    if skip {
        println!("{}", style("skipping post script!").yellow());
        return None
    }

    match MAP_COMMANDS_WITH_POST_SCRIPT.iter().find(|command_with_post_script| command_with_post_script.0 == *command) {
        Some(command_with_post_script) => Some(command_with_post_script.1),
        None => None
    }
}

fn run_post_script(script_name: &str, arg: &str) -> Result<(), String> {
    let script_path = Path::new(&get_data_path()?).join(POST_SCRIPTS_FOLDER_NAME).join(script_name);
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

fn get_data_path() -> Result<String, String> {
    let env_name = DATA_PATH_ENV_NAME;
    env::var(env_name).map_err(|_error|
        format!("please set '{env_name}' environment variable to point to manjaliof data folder"))
}
