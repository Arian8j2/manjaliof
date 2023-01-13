use vergen::{vergen, Config};

fn main() -> Result<(), String> {
    let mut config = Config::default();
    *config.git_mut().commit_message_mut() = true;
    *config.git_mut().sha_mut() = true;
    vergen(config).map_err(|e| e.to_string())
}
