use dialoguer::Validator;

const MAX_SHORT_STRING_LENGTH: usize = 64;

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

pub struct ShortStringValidator {}
impl Validator<String> for ShortStringValidator {
    type Err = String;

    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        if input.len() > MAX_SHORT_STRING_LENGTH {
            return Err(format!("max length of this field is {} characters", MAX_SHORT_STRING_LENGTH));
        }
        
        Ok(())
    }
}
