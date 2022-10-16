pub struct NumberValidator { }
impl dialoguer::Validator<String> for NumberValidator {
    type Err = String;
    fn validate(&mut self, input: &String) -> Result<(), Self::Err> {
        match input.parse::<u32>() {
            Ok(_) => Ok(()),
            Err(_) => Err("this field must be numeric".to_string())
        }
    }
}
