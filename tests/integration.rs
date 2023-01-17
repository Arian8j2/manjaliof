mod context;

use context::TestContext;
use indoc::indoc;

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

#[test]
fn set_info_should_fail_when_arguments_conflicts() {
    let context = TestContext::new();
    let error = "Error: --match-info and --all and --name conflicts with each other\n";
    context.run_command().args(&["set-info", "--all", "--name", "idk"])
        .assert().failure().stderr(error);
    context.run_command().args(&["set-info", "--name", "idk", "--match-info", "someinfo"])
        .assert().failure().stderr(error);
}

#[test]
fn set_info_match_info() {
    let context = TestContext::new();
    context.create_post_script("add", "#!/bin/bash");
    context.run_command().args(&["add", "--name", "testcase1",
                                 "--days", "30", "--seller", "pouya",
                                 "--money", "55", "--info", "idk"]).assert().success();
    context.run_command().args(&["add", "--name", "testcase2",
                                 "--days", "26", "--seller", "arian",
                                 "--money", "55", "--info", "nemidonam"]).assert().success();
    context.run_command().args(&["add", "--name", "testcase3",
                                 "--days", "29", "--seller", "arian",
                                 "--money", "60", "--info", "idk"]).assert().success();
    context.run_command().args(&["set-info", "--match-info", "idk",
                                 "--info", "newidk"]).assert().success();
    context.run_command().arg("list").assert().success().stdout(indoc! {"
        testcase1 29d pouya(55) newidk   
        testcase3 28d arian(60) newidk   
        testcase2 25d arian(55) nemidonam
    "});
}
