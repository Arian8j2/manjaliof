use assert_cmd::Command;
use rand::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct TestContext {
    data_path: PathBuf,
}

impl TestContext {
    pub fn new() -> TestContext {
        let random_indicator: u16 = random();
        let data_path_str = format!("/tmp/manjaliof-test-{random_indicator}");
        let data_path = Path::new(&data_path_str);

        fs::create_dir(&data_path).unwrap();
        fs::create_dir(data_path.join("post_scripts")).unwrap();

        TestContext {
            data_path: data_path.to_path_buf(),
        }
    }

    pub fn create_post_script(&self, post_script_name: &str, content: &str) {
        let script_path = &self.data_path.join("post_scripts").join(post_script_name);
        fs::write(script_path, content).unwrap();
        Command::new("chmod")
            .args(&["+x", script_path.to_str().unwrap()])
            .unwrap();
    }

    pub fn run_command(&self) -> Command {
        let mut cmd = Command::cargo_bin("manjaliof").unwrap();
        cmd.env("MANJALIOF_DATA", &self.data_path);
        cmd
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.data_path).unwrap();
    }
}
