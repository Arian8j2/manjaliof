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
    RenewAll,

    #[command(about="remove client")]
    Remove(RemoveArgs),

    #[command(about="show all clients")]
    List,

    #[command(about="rename client")]
    Rename,

    #[command(about="set client info")]
    SetInfo(SetInfoArgs),

    #[command(about="remove expired clients that are expired long time ago")]
    Cleanup
}

#[derive(Args, PartialEq)]
pub struct SetInfoArgs {
    #[arg(short, long, default_value_t = false)]
    pub all: bool
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
    pub info: Option<String>,
}

pub type RenewArgs = AddArgs;

#[derive(Args, PartialEq)]
pub struct RemoveArgs {
    #[arg(long)]
    pub name: Option<String>
}
