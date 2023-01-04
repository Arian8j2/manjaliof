mod cli;
mod db;
mod input;
mod report;

use cli::{Commands, Cli, AddArgs, RenewArgs, RemoveArgs, SetInfoArgs};
use clap::Parser;
use std::{env, process, process::ExitCode, path::Path};
use db::{Database, BackupableDatabase, jsondb::JsonDb, Target};
use dialoguer::console::style;
use report::{Report, client_report};
use chrono::Utc;

type PostScriptArgs = Option<Vec<String>>;

const DATA_PATH_ENV_NAME: &str = "MANJALIOF_DATA";
const DB_FILE_NAME: &str = "data.json";
const POST_SCRIPTS_FOLDER_NAME: &str = "post_scripts";

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
    let cli = Cli::parse();

    let db_path = Path::new(&get_data_path()?).join(DB_FILE_NAME);
    let db = JsonDb::new(db_path.to_str().unwrap());
    let backup = db.get_backup()?;

    let command_result = try_run_command(&cli, &db);
    if command_result.is_err() && cli.command != Commands::List {
        db.restore_backup(backup)?;
    }

    command_result
}

fn try_run_command(cli: &Cli, db: &dyn Database) -> Result<(), String> {
    let post_script_name = get_command_post_script(&cli.command, cli.skip_post_script);
    let post_script_arg = match &cli.command {
        Commands::Add(args) => add_client(db, args)?,
        Commands::Renew(args) => renew_client(db, args)?,
        Commands::RenewAll => renew_all_clients(db)?,
        Commands::Remove(args) => remove_client(db, args)?,
        Commands::List => list_clients(db)?,
        Commands::Rename => rename_client(db)?,
        Commands::SetInfo(args) => set_client_info(db, &args)?,
        Commands::Cleanup => cleanup(db)?
    };

    if let (Some(name), Some(args)) = (post_script_name, post_script_arg){
        run_post_script(name, args)?;
    }
    
    Ok(())
}

fn add_client(db: &dyn Database, args: &AddArgs) -> Result<PostScriptArgs, String> {
    let name = args.name.clone().unwrap_or_else(input::get_client_name);
    let days = args.days.unwrap_or_else(input::get_days);
    let seller = args.seller.clone().unwrap_or_else(input::get_seller);
    let money = args.money.unwrap_or_else(input::get_money_amount);
    let info = args.info.clone().unwrap_or_else(|| input::get_info(None));

    input::validators::validate_name(&name)?;
    input::validators::validate_seller(&seller)?;
    input::validators::validate_info(&info)?;

    db.add_client(&name, days, &seller, money, &info)?;
    Ok(Some(vec![name]))
}

fn renew_client(db: &dyn Database, args: &RenewArgs) -> Result<PostScriptArgs, String> {
    let name = args.name.clone().unwrap_or_else(input::get_client_name);
    let days = args.days.unwrap_or_else(input::get_days);
    let seller = args.seller.clone().unwrap_or_else(input::get_seller);
    let money = args.money.unwrap_or_else(input::get_money_amount);
    let mut info = args.info.clone().unwrap_or(String::new());

    if info.is_empty() {
        let last_info = db.get_client_info(&name)?;
        info = input::get_info(Some(&last_info));
    }

    input::validators::validate_name(&name)?;
    input::validators::validate_seller(&seller)?;
    input::validators::validate_info(&info)?;

    db.renew_client(&name, days, &seller, money)?;
    db.set_client_info(Target::OnePerson(name.clone()), &info)?;
    Ok(Some(vec![name]))
}

fn renew_all_clients(db: &dyn Database) -> Result<PostScriptArgs, String> {
    println!("{}", style("you are renewing all clients that are not expired!").yellow());
    let days = input::get_days();
    db.renew_all_clients(days)?;
    Ok(None)
}

fn remove_client(db: &dyn Database, args: &RemoveArgs) -> Result<PostScriptArgs, String> {
    let name = args.name.clone().unwrap_or_else(input::get_client_name);
    input::validators::validate_name(&name)?;
    db.remove_client(&name)?;
    Ok(Some(vec![name]))
}

fn list_clients(db: &dyn Database) -> Result<PostScriptArgs, String> {
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

fn rename_client(db: &dyn Database) -> Result<PostScriptArgs, String> {
    let old_name = input::get_client_name();
    let new_name = input::get_client_new_name();
    db.rename_client(&old_name, &new_name)?;
    Ok(Some(vec![old_name, new_name]))
}

fn set_client_info(db: &dyn Database, args: &SetInfoArgs) -> Result<PostScriptArgs, String> {
    let target: Target = if args.all { Target::All } else { Target::OnePerson(input::get_client_name()) };
    let last_info = match &target {
        Target::OnePerson(name) => db.get_client_info(&name)?,
        Target::All => "".to_string()
    };

    let new_info = input::get_info(Some(&last_info));
    db.set_client_info(target, &new_info)?;
    Ok(None)
}

fn cleanup(db: &dyn Database) -> Result<PostScriptArgs, String> {
    let now_time = Utc::now();

    let clients = db.list_clients()?;
    for client in clients {
        let is_expired_ten_days_ago = (now_time - client.expire_time).num_days() >= 10;
        if is_expired_ten_days_ago {
            db.remove_client(&client.name)?;
            run_post_script("delete", vec![client.name.clone()])?;
            println!("{}", style(format!("deleted {}", client.name)).yellow());
        }
    }

    Ok(None)
}

fn get_command_post_script(command: &Commands, skip: bool) -> Option<&'static str> {
    if skip {
        println!("{}", style("skipping post script!").yellow());
        return None
    }

    match &command {
        Commands::Add(_) => Some("add"),
        Commands::Renew(_) => Some("renew"),
        Commands::Remove(_) => Some("delete"),
        Commands::Rename => Some("rename"),
        _ => None
    }
}

fn run_post_script(script_name: &str, args: Vec<String>) -> Result<(), String> {
    let script_path = Path::new(&get_data_path()?).join(POST_SCRIPTS_FOLDER_NAME).join(script_name);
    let output = process::Command::new(&script_path).args(args).output().map_err(
        |error| format!("couldn't run post script '{}': {}", script_path.to_str().unwrap(), error.to_string()))?;

    if !output.status.success() {
        let mut output_stderr = std::str::from_utf8(output.stderr.as_slice()).unwrap().to_string();
        if let Some(last_char) = output_stderr.chars().last() {
            if last_char == '\n' {
                output_stderr.pop();
            }
        }

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
