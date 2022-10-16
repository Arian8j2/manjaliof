mod datetime_serializer;
pub mod jsondb;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const DAYS_PER_MONTH: u32 = 30;

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

    pub payments: Vec<Payment>
}

impl Client {
    fn new(name: &str, months: u32, seller: &str, money: u32) -> Client {
        let now_date = Utc::now();
        let expire_time = now_date + Duration::days((months * DAYS_PER_MONTH).into());

        Client {
            name: name.to_string(),
            expire_time,
            payments: vec![Payment { seller: seller.to_string(), money, date: now_date }],
        }
    }
}

pub trait Database {
    fn add_client(&self, name: &str, months: u32, seller: &str, money: u32) -> Result<(), String>;
    fn renew_client(&self, name: &str, months: u32, seller: &str, money: u32) -> Result<(), String>;
    fn delete_client(&self, name: &str) -> Result<(), String>;
    fn list_clients(&self) -> Result<Vec<Client>, String>;
}
