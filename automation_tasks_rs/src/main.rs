//! automation_tasks_rs with_lib

use cargo_auto_lib::*;

/// automation_tasks_rs with_lib
fn main() {
    exit_if_not_run_in_rust_project_root_directory();

    // get CLI arguments
    let mut args = std::env::args();
    // the zero argument is the name of the program
    let _arg_0 = args.next();
    match_arguments_and_call_tasks(args);
}

// region: match, help and completion. Take care to keep them in sync with the changes.

/// match arguments and call tasks functions
fn match_arguments_and_call_tasks(mut args: std::env::Args) {
    // the first argument is the user defined task: (no argument for help), build, release,...
    let arg_1 = args.next();
    match arg_1 {
        None => print_help(),
        Some(task) => {
            if &task == "completion" {
                completion();
            } else {
                println!("Running automation task: {}", &task);
                if &task == "build" || &task == "b" {
                    task_build();
                } else if &task == "release" || &task == "r" {
                    task_release();
                } else if &task == "increment_minor" {
                    task_increment_minor();
                } else if &task == "docs" || &task == "doc" || &task == "d" {
                    task_docs();
                } else if &task == "publish_to_crates_io" {
                    task_publish_to_crates_io();
                } else {
                    println!("Task {} is unknown.", &task);
                    print_help();
                }
            }
        }
    }
}

/// write a comprehensible help for user defined tasks
fn print_help() {
    println!(
        r#"
User defined tasks in automation_tasks_rs:
cargo auto build - builds the crate in debug mode, fmt
cargo auto release - builds the crate in release mode, version from date, fmt"
cargo auto increment_minor - increments the semver version minor part (only for libraries)
cargo auto docs - builds the docs, copy to docs directory
cargo auto publish_to_crates_io - publish to crates.io, git tag
"#
    );
}

/// sub-command for bash auto-completion of `cargo auto` using the crate `dev_bestia_cargo_completion`
fn completion() {
    let args: Vec<String> = std::env::args().collect();
    let word_being_completed = args[2].as_str();
    let last_word = args[3].as_str();

    if last_word == "cargo-auto" || last_word == "auto" {
        let sub_commands = vec!["build", "release", "doc", "publish_to_crates_io"];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
    /*
    // the second level if needed
    else if last_word == "new" {
        let sub_commands = vec!["with_lib"];
        completion_return_one_or_more_sub_commands(sub_commands, word_being_completed);
    }
    */
}

// endregion: match, help and completion.

// region: tasks

/// example how to call a list of shell commands
fn task_build() {
    #[rustfmt::skip]
    let shell_commands = [
        "cargo fmt", 
        "cargo build"];
    run_shell_commands(shell_commands.to_vec());
    println!(
        r#"
After `cargo auto build`, run the tests and the code. If ok, then `cargo auto release`
"#
    );
}

/// example how to call one shell command and combine with rust code
fn task_release() {
    auto_semver_increment_patch();
    auto_cargo_toml_to_md();
    auto_lines_of_code("");

    run_shell_command("cargo fmt");
    run_shell_command("cargo build --release");
    println!(
        r#"
After `cargo auto release`, run the tests and the code. If ok, then `cargo auto doc`
"#
    );
}

/// semver is used for libraries, version_from_date is used for binary
fn task_increment_minor() {
    auto_semver_increment_minor();
    auto_cargo_toml_to_md();
}

/// example how to call a list of shell commands and combine with rust code
fn task_docs() {
    auto_md_to_doc_comments();
    #[rustfmt::skip]
    let shell_commands = [
        "cargo doc --no-deps --document-private-items --open",        
        // copy target/doc into docs/ because it is github standard
        "rsync -a --info=progress2 --delete-after target/doc/ docs/",
        "echo Create simple index.html file in docs directory",
        &format!("echo \"<meta http-equiv=\\\"refresh\\\" content=\\\"0; url={}/index.html\\\" />\" > docs/index.html",package_name().replace("-","_")) ,
    ];
    run_shell_commands(shell_commands.to_vec());
    // message to help user with next move
    println!(
        r#"
After `cargo auto doc`, check `docs/index.html`. If ok, then `git commit -am"message"` and `git push`,
then `cargo auto publish_to_crates_io`
"#
    );
}

/// example hot to publish to crates.io and git tag
fn task_publish_to_crates_io() {
    // git tag
    let shell_command = format!(
        "git tag -f -a v{version} -m version_{version}",
        version = package_version()
    );
    run_shell_command(&shell_command);

    // cargo publish
    run_shell_command("cargo publish");
    println!(
        r#"
After `cargo auto task_publish_to_crates_io', check `crates.io` page.
If binary then install with `cargo install {package_name}` and check how it works.
If library then add dependency `{package_name} = "{package_version}"` to your rust project and check how it works.
"#,
        package_name = package_name(),
        package_version = package_version()
    );
}

// endregion: tasks

// region: helper functions

/// check if run in rust project root directory error
/// there must be Cargo.toml and the directory automation_tasks_rs
/// exit with error message if not
fn exit_if_not_run_in_rust_project_root_directory() {
    if !(std::path::Path::new("automation_tasks_rs").exists()
        && std::path::Path::new("Cargo.toml").exists())
    {
        println!("Error: automation_tasks_rs must be called in the root directory of the rust project beside the Cargo.toml file and automation_tasks_rs directory.");
        // early exit
        std::process::exit(0);
    }
}

/// println one, more or all sub_commands
fn completion_return_one_or_more_sub_commands(sub_commands: Vec<&str>, word_being_completed: &str) {
    let mut sub_found = false;
    for sub_command in sub_commands.iter() {
        if sub_command.starts_with(word_being_completed) {
            println!("{}", sub_command);
            sub_found = true;
        }
    }
    if sub_found == false {
        // print all sub-commands
        for sub_command in sub_commands.iter() {
            println!("{}", sub_command);
        }
    }
}

// endregion: helper functions
