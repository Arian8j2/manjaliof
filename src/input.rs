mod validators;

use dialoguer::{Input, Select, theme, console::style};

const REFFERS: [&'static str; 2] = ["arian", "pouya"];

pub fn get_client_name() -> String {
    Input::with_theme(&get_theme()).with_prompt("client name").interact_text().unwrap()
}

pub fn get_seller() -> String {
    let reffer_index: usize = Select::with_theme(&get_theme()).with_prompt("who gets money")
                                                              .items(&REFFERS).interact().unwrap();
    REFFERS.get(reffer_index).unwrap().to_string()
}

pub fn get_money_amount() -> u32 {
    Input::with_theme(&get_theme()).with_prompt("money money")
                                   .default("40".into())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

pub fn get_months() -> u32 {
    Input::with_theme(&get_theme()).with_prompt("how many months")
                                   .default("1".into())
                                   .validate_with(validators::NumberValidator {})
                                   .interact_text().unwrap().parse().unwrap()
}

fn get_theme() -> impl theme::Theme {
    let mut theme = theme::ColorfulTheme::default();
    theme.success_prefix = style("✓".to_string()).for_stderr().green();
    theme.checked_item_prefix = style("✓".to_string()).for_stderr().green();
    theme.unchecked_item_prefix = style("✓".to_string()).for_stderr().black();
    theme
}
