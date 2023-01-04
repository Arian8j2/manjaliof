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
            Err(_) => Err("this field must be numeric".to_string())
        }
    }
}

pub fn validate_name(name: &String) -> Result<(), String> {
    if name.contains(' ') || name.len() > MAX_NAME_LENGTH || !name.is_ascii() {
        return Err(format!("cannot validate name: it must be ascii and not contain \
                            space and be less than equal to {} characters.", MAX_NAME_LENGTH));
    }
    Ok(())
}

pub fn validate_seller(seller: &String) -> Result<(), String> {
    if !SELLERS.contains(&seller.as_str()) {
        return Err(format!("cannot validate seller: only this sellers are valid: {}", SELLERS.join(", ")));
    }
    Ok(())
}

pub fn validate_info(info: &String) -> Result<(), String> {
    if info.len() > MAX_INFO_LENGTH {
        return Err(format!("cannot validate info: max length of info is {}", MAX_INFO_LENGTH));
    }
    Ok(())
}
