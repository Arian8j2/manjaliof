mod datetime_serializer;
pub mod jsondb;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Payment {
    pub seller: String,
    pub money: u32,

    #[serde(with="datetime_serializer")]
    pub date: DateTime<Utc>
}

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub name: String,

    #[serde(with="datetime_serializer")]
    pub expire_time: DateTime<Utc>,

    pub payments: Vec<Payment>,

    pub info: Option<String>
}

impl Client {
    fn new(name: &str, days: u32, seller: &str, money: u32, info: &str) -> Client {
        let now_date = Utc::now();
        let expire_time = now_date + Duration::days(days.into());

        Client {
            name: name.to_string(),
            expire_time,
            payments: vec![Payment { seller: seller.to_string(), money, date: now_date }],
            info: Some(info.to_string())
        }
    }
}

pub enum Target {
    All,
    OnePerson(String)
}

pub trait Database {
    fn add_client(&self, name: &str, days: u32, seller: &str, money: u32, info: &str) -> Result<(), String>;
    fn renew_client(&self, name: &str, days: u32, seller: &str, money: u32) -> Result<(), String>;
    fn renew_all_clients(&self, days: u32) -> Result<(), String>;
    fn remove_client(&self, name: &str) -> Result<(), String>;
    fn list_clients(&self) -> Result<Vec<Client>, String>;
    fn rename_client(&self, old_name: &str, new_name: &str) -> Result<(), String>;
    fn set_client_info(&self, target: Target, info: &str) -> Result<(), String>;
    fn get_client_info(&self, name: &str) -> Result<String, String>;
}

pub trait BackupableDatabase {
    type DbData;

    fn get_backup(&self) -> Result<Self::DbData, String>;
    fn restore_backup(&self, backup: Self::DbData) -> Result<(), String>;
}
