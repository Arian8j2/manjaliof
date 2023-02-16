mod cli;
mod db;
mod input;
mod report;

use chrono::Utc;
use clap::Parser;
use cli::{
    AddArgs, Cli, Commands, EditArgs, ListArgs, RemoveArgs, RenameArgs, RenewAllArgs, RenewArgs,
    SetInfoArgs,
};
use db::{sqlitedb::SqliteDb, Database, Target};
use dialoguer::console::style;
use report::{client_report, Report};
use std::{env, path::Path, process, process::ExitCode};

type PostScriptArgs = Option<Vec<String>>;

const DATA_PATH_ENV_NAME: &str = "MANJALIOF_DATA";
const DB_FILE_NAME: &str = "data.db";
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
    let mut conn = SqliteDb::create_connection(db_path)?;
    let mut db = SqliteDb::new(&mut conn)?;

    let command_result = try_run_command(&cli, &mut db);
    if !command_result.is_err() {
        db.commit()
            .map_err(|e| format!("CRITICAL ERROR: cannot commit changes: {e}"))?;
    }

    command_result
}

fn try_run_command<T: Database>(cli: &Cli, db: &mut T) -> Result<(), String> {
    let post_script_name = get_command_post_script(&cli.command, cli.skip_post_script);
    let post_script_arg = match &cli.command {
        Commands::Add(args) => add_client(db, args)?,
        Commands::Renew(args) => renew_client(db, args)?,
        Commands::RenewAll(args) => renew_all_clients(db, args)?,
        Commands::Edit(args) => edit_client(db, args)?,
        Commands::Remove(args) => remove_client(db, args)?,
        Commands::List(args) => list_clients(db, args)?,
        Commands::Rename(args) => rename_client(db, args)?,
        Commands::SetInfo(args) => set_client_info(db, &args)?,
        Commands::Cleanup => cleanup(db)?,
        Commands::Version => version(),
    };

    if let (Some(name), Some(args)) = (post_script_name, post_script_arg) {
        run_post_script(name, args)?;
    }

    Ok(())
}

fn add_client<T: Database>(db: &mut T, args: &AddArgs) -> Result<PostScriptArgs, String> {
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

fn renew_client<T: Database>(db: &mut T, args: &RenewArgs) -> Result<PostScriptArgs, String> {
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

fn renew_all_clients<T: Database>(
    db: &mut T,
    args: &RenewAllArgs,
) -> Result<PostScriptArgs, String> {
    println!(
        "{}",
        style("you are renewing all clients that are not expired!").yellow()
    );
    let days = args.days.unwrap_or_else(input::get_days);
    db.renew_all_clients(days)?;
    Ok(None)
}

fn edit_client<T: Database>(db: &mut T, args: &EditArgs) -> Result<PostScriptArgs, String> {
    let name = args.name.clone().unwrap_or_else(input::get_client_name);
    let client = db
        .list_clients()?
        .into_iter()
        .find(|client| client.name == name)
        .ok_or(format!("client with name '{name}' doesn't exists!"))?;

    let now_time = Utc::now();
    let days_remain = (client.expire_time - now_time).num_days();
    if days_remain < 0 {
        return Err("cannot edit an expired client".to_string());
    }
    let days = args
        .days
        .unwrap_or_else(|| input::get_new_days(days_remain.try_into().unwrap()));

    let last_payment = client.payments.last().unwrap();
    let seller = args
        .seller
        .clone()
        .unwrap_or_else(|| input::get_new_seller(&last_payment.seller));
    let money = args
        .money
        .unwrap_or_else(|| input::get_new_money_amount(last_payment.money));
    input::validators::validate_seller(&seller)?;

    let last_info = client.info.unwrap_or("".to_string());
    let info = args
        .info
        .clone()
        .unwrap_or_else(|| input::get_info(Some(&last_info)));
    input::validators::validate_info(&info)?;

    db.edit_client(&name, days, &seller, money, &info)?;
    Ok(None)
}

fn remove_client<T: Database>(db: &mut T, args: &RemoveArgs) -> Result<PostScriptArgs, String> {
    let name = args.name.clone().unwrap_or_else(input::get_client_name);
    input::validators::validate_name(&name)?;
    db.remove_client(&name)?;
    Ok(Some(vec![name]))
}

fn list_clients<T: Database>(db: &mut T, args: &ListArgs) -> Result<PostScriptArgs, String> {
    let mut clients = db.list_clients()?;
    clients.sort_by_key(|client| client.expire_time);
    clients.reverse();

    let mut report = Report::new(["name", "months left", "seller", "info"].to_vec());
    for client in clients {
        let name = style(client.name).cyan().to_string();
        let days_left = client_report::calculate_days_left(args.verbose, client.expire_time);
        let sellers = client_report::calculate_sellers(&client.payments);
        let info = style(client.info.unwrap_or("".to_string()))
            .black()
            .bright()
            .to_string();

        report.add_item([name, days_left, sellers, info].to_vec());
    }

    report.show(args.trim_whitespace);
    Ok(None)
}

fn rename_client<T: Database>(db: &mut T, args: &RenameArgs) -> Result<PostScriptArgs, String> {
    let old_name = args.old_name.clone().unwrap_or_else(input::get_client_name);
    let new_name = args
        .new_name
        .clone()
        .unwrap_or_else(input::get_client_new_name);

    input::validators::validate_name(&new_name)?;

    db.rename_client(&old_name, &new_name)?;
    Ok(Some(vec![old_name, new_name]))
}

fn set_client_info<T: Database>(db: &mut T, args: &SetInfoArgs) -> Result<PostScriptArgs, String> {
    if (args.all as i32) + (args.match_info.is_some() as i32) + (args.name.is_some() as i32) > 1 {
        return Err("--match-info and --all and --name conflicts with each other".to_string());
    }

    let target: Target = if args.all {
        Target::All
    } else if let Some(old_info) = &args.match_info {
        Target::MatchInfo(old_info.clone())
    } else {
        Target::OnePerson(args.name.clone().unwrap_or_else(input::get_client_name))
    };

    let last_info = match &target {
        Target::All => "".to_string(),
        Target::MatchInfo(old_info) => old_info.clone(),
        Target::OnePerson(name) => db.get_client_info(&name)?,
    };
    let new_info = args
        .info
        .clone()
        .unwrap_or_else(|| input::get_info(Some(&last_info)));
    db.set_client_info(target, &new_info)?;

    Ok(None)
}

fn cleanup<T: Database>(db: &mut T) -> Result<PostScriptArgs, String> {
    let now_time = Utc::now();

    let clients = db.list_clients()?;
    for client in clients {
        let is_expired_five_days_ago = (now_time - client.expire_time).num_days() >= 5;
        if is_expired_five_days_ago {
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
        return None;
    }

    match &command {
        Commands::Add(_) => Some("add"),
        Commands::Renew(_) => Some("renew"),
        Commands::Remove(_) => Some("delete"),
        Commands::Rename(_) => Some("rename"),
        _ => None,
    }
}

fn run_post_script(script_name: &str, args: Vec<String>) -> Result<(), String> {
    let script_path = Path::new(&get_data_path()?)
        .join(POST_SCRIPTS_FOLDER_NAME)
        .join(script_name);
    let output = process::Command::new(&script_path)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "couldn't run post script '{}': {}",
                script_path.to_str().unwrap(),
                error.to_string()
            )
        })?;

    if !output.status.success() {
        let mut output_stderr = std::str::from_utf8(output.stderr.as_slice())
            .unwrap()
            .to_string();
        if let Some(last_char) = output_stderr.chars().last() {
            if last_char == '\n' {
                output_stderr.pop();
            }
        }

        return Err(format!(
            "post script exited due to a failure: {}",
            output_stderr
        ));
    }

    let result = std::str::from_utf8(output.stdout.as_slice())
        .unwrap()
        .to_string();
    if !result.is_empty() {
        println!("{}", result);
    }
    Ok(())
}

fn get_data_path() -> Result<String, String> {
    let env_name = DATA_PATH_ENV_NAME;
    env::var(env_name).map_err(|_error| {
        format!("please set '{env_name}' environment variable to point to manjaliof data folder")
    })
}

fn version() -> PostScriptArgs {
    println!(
        "{}\n{}",
        style(env!("VERGEN_GIT_SHA")).green(),
        env!("VERGEN_GIT_COMMIT_MESSAGE")
    );
    None
}
