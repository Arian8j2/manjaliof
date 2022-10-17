use chrono::{DateTime, Utc};
use dialoguer::console::style;
use std::collections::HashMap;
use crate::db::Payment;

pub fn calculate_months_left(expire_time: DateTime<Utc>) -> String {
    let now_date = Utc::now();
    if expire_time < now_date {
        return style("expired").red().to_string();
    }

    let delta = expire_time - now_date;
    let num_months: u8 = (delta.num_days() as f32 / 30.0).round() as u8;
    if num_months == 0 {
        return style(format!("{}d", delta.num_days())).yellow().to_string();
    }

    style(format!("{num_months}m")).green().to_string()
}

pub fn calculate_sellers(payments: &Vec<Payment>) -> String {
    let mut sellers: HashMap<&str, u32> = HashMap::new();
    for payment in payments {
        let entry = sellers.entry(&payment.seller).or_insert(0);
        *entry += payment.money;
    }

    let mut sellers_string: Vec<String> = Vec::new();
    for seller in sellers {
        let seller_string = format!("{}({})", seller.0, seller.1);
        sellers_string.push(seller_string);
    }

    sellers_string.join(",")
}

