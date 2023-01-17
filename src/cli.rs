use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about="this program will always remain manjaliof")]
pub struct Cli {
    #[command(subcommand)] 
    pub command: Commands,

    #[arg(long, default_value_t = false)]
    pub skip_post_script: bool
}

#[derive(Subcommand, PartialEq)]
pub enum Commands {
    #[command(about="adds new client to db")]
    Add(AddArgs),

    #[command(about="renew client")]
    Renew(RenewArgs),

    #[command(about="renew all clients that are not expired")]
    RenewAll(RenewAllArgs),

    #[command(about="remove client")]
    Remove(RemoveArgs),

    #[command(about="show all clients")]
    List(ListArgs),

    #[command(about="rename client")]
    Rename(RenameArgs),

    #[command(about="set client info")]
    SetInfo(SetInfoArgs),

    #[command(about="remove expired clients that are expired long time ago")]
    Cleanup,

    #[command(about="show message and sha256 of latest commit that is built from")]
    Version
}

#[derive(Args, PartialEq)]
pub struct AddArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub days: Option<u32>,

    #[arg(long)]
    pub seller: Option<String>,

    #[arg(long)]
    pub money: Option<u32>,

    #[arg(long)]
    pub info: Option<String>
}

pub type RenewArgs = AddArgs;

#[derive(Args, PartialEq)]
pub struct RenewAllArgs {
    #[arg(long)]
    pub days: Option<u32>
}

#[derive(Args, PartialEq)]
pub struct RemoveArgs {
    #[arg(long)]
    pub name: Option<String>
}

#[derive(Args, PartialEq)]
pub struct ListArgs {
    #[arg(long, default_value_t = false)]
    pub trim_whitespace: bool
}

#[derive(Args, PartialEq)]
pub struct RenameArgs {
    #[arg(long)]
    pub old_name: Option<String>,

    #[arg(long)]
    pub new_name: Option<String>
}

#[derive(Args, PartialEq)]
pub struct SetInfoArgs {
    #[arg(long, default_value_t = false)]
    pub all: bool,

    #[arg(long)]
    pub match_info: Option<String>,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub info: Option<String>
}
