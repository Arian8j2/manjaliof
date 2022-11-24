use chrono::{DateTime, Utc};
use dialoguer::console::style;
use crate::db::Payment;

pub fn calculate_days_left(expire_time: DateTime<Utc>) -> String {
    let now_date = Utc::now();
    if expire_time < now_date {
        return style("expired").red().to_string();
    }

    let delta = expire_time - now_date;
    let num_days = delta.num_days();
    if num_days < 15 {
        return style(format!("{num_days}d")).yellow().to_string();
    }

    style(format!("{num_days}d")).green().to_string()
}

pub fn calculate_sellers(payments: &Vec<Payment>) -> String {
    assert!(!payments.is_empty());
    let last_payment = payments.iter().rev().next().unwrap();
    format!("{}({})", last_payment.seller, last_payment.money)
}

