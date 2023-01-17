use assert_cmd::Command;
use rand::prelude::*;
use std::{fs, path::{Path, PathBuf}};
use indoc::indoc;

struct TestContext {
    data_path: PathBuf
}

impl TestContext {
    fn new() -> TestContext {
        let random_indicator: u16 = random();
        let data_path_str = format!("/tmp/manjaliof-test-{random_indicator}");
        let data_path = Path::new(&data_path_str);

        fs::create_dir(&data_path).unwrap();
        fs::create_dir(data_path.join("post_scripts")).unwrap();

        TestContext { data_path: data_path.to_path_buf() }
    }

    fn create_post_script(&self, post_script_name: &str, content: &str) {
        let script_path = &self.data_path.join("post_scripts").join(post_script_name);
        fs::write(script_path, content).unwrap();
        Command::new("chmod").args(&["+x", script_path.to_str().unwrap()]).unwrap();
    }

    fn run_command(&self) -> Command {
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

#[test]
fn add() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.run_command().arg("list").assert().success().stdout("testcase 29d pouya(60) idk\n");
}

#[test]
fn should_reset_db_when_add_post_script_failed() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash\nexit 1");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().failure();
    context.run_command().arg("list").assert().success().stdout("");
}

#[test]
fn list_with_trimmed_whitespace() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.run_command().args(&["add", "--name", "testcasewithlongname",
                                 "--days", "19", "--seller", "arian",
                                 "--money", "50", "--info", "nemidonam"]).assert().success();
    context.run_command().args(&["list", "--trim-whitespace"]).assert().success().stdout(indoc! {"
        testcase 29d pouya(60) idk
        testcasewithlongname 18d arian(50) nemidonam
    "});
}

#[test]
fn renew() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcaserenew",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.create_post_script("renew", "#!/bin/bash");
    context.run_command().args(&["renew", "--name", "testcaserenew",
                                 "--days", "10", "--seller", "arian",
                                 "--money", "30", "--info", "smth"]).assert().success();
    context.run_command().arg("list").assert().success().stdout("testcaserenew 39d arian(30) smth\n");
}

#[test]
fn renew_all() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase2",
                                 "--days", "30", "--seller", "arian",
                                 "--money", "55", "--info", "smth"]).assert().success();
    context.run_command().args(&["add", "--name", "testcase1",
                                 "--days", "20", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.run_command().args(&["renew-all", "--days", "10"]).assert().success();
    context.run_command().arg("list").assert().success().stdout(indoc! {"
        testcase2 39d arian(55) smth
        testcase1 29d pouya(60) idk 
    "});
}

#[test]
fn remove() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.create_post_script("delete", "#!/bin/bash");
    context.run_command().args(&["remove", "--name", "testcase"]).assert().success();
    context.run_command().arg("list").assert().success().stdout("");
}

#[test]
fn rename() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.create_post_script("rename", "#!/bin/bash");
    context.run_command().args(&["rename", "--old-name", "testcase",
                                 "--new-name", "testcasenew"]).assert().success();
    context.run_command().arg("list").assert().success().stdout("testcasenew 29d pouya(60) idk\n");
}

#[test]
fn set_info() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.run_command().args(&["set-info", "--name", "testcase",
                                 "--info", "newinfo"]).assert().success();
    context.run_command().arg("list").assert().success().stdout("testcase 29d pouya(60) newinfo\n");
}
