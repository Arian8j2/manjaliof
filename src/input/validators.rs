use dialoguer::Validator;

use super::SELLERS;

const MAX_NAME_LENGTH: usize = 35;
const MAX_INFO_LENGTH: usize = 64;

pub struct NumberValidator {}
impl Validator<String> for NumberValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        match input.parse::<u32>() {
            Ok(_) => Ok(()),
            Err(_) => Err("this field must be numeric".to_string()),
        }
    }
}

pub fn validate_name(name: &String) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_NAME_LENGTH {
        return Err("cannot validate name: text is too short or too long".to_string());
    }

    if !name
        .chars()
        .all(|ch| char::is_ascii_alphanumeric(&ch) || ch == '-')
    {
        return Err("cannot validate name: only ascii alphanumeric values are valid".to_string());
    }

    Ok(())
}

pub fn validate_seller(seller: &String) -> Result<(), String> {
    if !SELLERS.contains(&seller.as_str()) {
        return Err(format!(
            "cannot validate seller: only this sellers are valid: {}",
            SELLERS.join(", ")
        ));
    }
    Ok(())
}

pub fn validate_info(info: &String) -> Result<(), String> {
    if info.is_empty() || info.len() > MAX_INFO_LENGTH {
        return Err(format!(
            "cannot validate info: text is too short or too long"
        ));
    }
    Ok(())
}
