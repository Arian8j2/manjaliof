pub mod validators;

use dialoguer::{Input, Select, theme, console::style};

pub const SELLERS: [&'static str; 2] = ["arian", "pouya"];

pub fn get_client_name() -> String {
    Input::with_theme(&get_theme()).with_prompt("client name").interact_text().unwrap()
}

pub fn get_client_new_name() -> String {
    Input::with_theme(&get_theme()).with_prompt("client new name").interact_text().unwrap()
}

pub fn get_seller() -> String {
    let reffer_index: usize = Select::with_theme(&get_theme()).with_prompt("who gets money")
                                                              .items(&SELLERS).interact().unwrap();
    SELLERS.get(reffer_index).unwrap().to_string()
}

pub fn get_new_seller(old_seller: &str) -> String {
    let old_seller = SELLERS.iter().position(|&x| x == old_seller).unwrap();
    let reffer_index: usize = Select::with_theme(&get_theme()).with_prompt("who gets money")
                                                              .default(old_seller)
                                                              .items(&SELLERS).interact().unwrap();
    SELLERS.get(reffer_index).unwrap().to_string()
}

pub fn get_money_amount() -> u32 {
    Input::with_theme(&get_theme()).with_prompt("money money")
                                   .default("60".into())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

pub fn get_new_money_amount(old_money_amount: u32) -> u32 {
    Input::with_theme(&get_theme()).with_prompt("money money")
                                   .with_initial_text(old_money_amount.to_string())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

pub fn get_days() -> u32 {
    Input::with_theme(&get_theme()).with_prompt("how many days")
                                   .default("30".into())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

pub fn get_new_days(old_days: u32) -> u32 {
    Input::with_theme(&get_theme()).with_prompt("how many days")
                                   .with_initial_text(old_days.to_string())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

pub fn get_info(last_info: Option<&str>) -> String {
    Input::with_theme(&get_theme()).with_prompt("extra info")
                                   .allow_empty(true)
                                   .with_initial_text(last_info.unwrap_or(""))
                                   .interact_text().unwrap()
}

fn get_theme() -> impl theme::Theme {
    let mut theme = theme::ColorfulTheme::default();
    theme.success_prefix = style("✓".to_string()).for_stderr().green();
    theme.checked_item_prefix = style("✓".to_string()).for_stderr().green();
    theme.unchecked_item_prefix = style("✓".to_string()).for_stderr().black();
    theme
}
