// auto_crates_io_mod.rs

use crate::auto_ssh_mod::SecretString;
use crate::RED;
use crate::RESET;
use crate::YELLOW;
// bring the trait into scope
use zeroize::Zeroize;

// file contains crates.io token encrypted with github_com_ssh_1
pub const CRATES_IO_TOKEN_PATH: &str = "~/.ssh/crates_io_data_1.ssh";

/// encrypt/decrypt the crates.io token with the GitHub ssh key
/// then call the `cargo publish --token token` command
/// but never show the secret token anywhere
pub fn publish_to_crates_io_with_secret_token() {
    let mut token = check_or_get_crates_io_token().unwrap();
    // don't show the token to the user
    println!("cargo publish with token");
    let shell_command = format!("cargo publish --token {}", token.0);
    let status = std::process::Command::new("sh").arg("-c").arg(shell_command).spawn().unwrap().wait().unwrap();
    token.0.zeroize();
    let exit_code = status.code();
    if exit_code.is_none() || exit_code != Some(0) {
        eprintln!("{RED}!!! cargo_auto publish error {}. Stopping automation task execution !!!{RESET}", exit_code.unwrap());
        std::process::exit(1);
    }
}

/// decrypt the token from CRATES_IO_TOKEN_PATH file
/// or ask user interactively to type it, then encrypt and save to file
fn check_or_get_crates_io_token() -> Option<SecretString> {
    // ssh_add_resolve(host_name: &str, default_identity_file_path: &str)
    let (_fingerprint, identity_file_path) = crate::auto_github_mod::ssh_add_resolve("github.com", "~/.ssh/github_com_ssh_1").unwrap();

    let mut token: Option<SecretString> = None;
    let crates_io_token_path_expanded = crate::utils_mod::file_path_home_expand(CRATES_IO_TOKEN_PATH);
    if std::path::Path::new(&crates_io_token_path_expanded).exists() {
        token = crate::auto_encrypt_decrypt_with_ssh_mod::decrypt_with_ssh_from_file(&crates_io_token_path_expanded);
    }
    if token.is_none() {
        println!(
            r#"{RED}Cannot find the file with encrypted crates.io token.{RESET}
    {YELLOW}The token is required to publish to crates.io.
    You can generate the token at https://crates.io/settings/tokens.
    The token is a secret just like a password, use it with caution.{RESET}
"#
        );
        // encrypt and save to file
        crate::auto_encrypt_decrypt_with_ssh_mod::encrypt_with_ssh_interactive_save_file(&identity_file_path, &crates_io_token_path_expanded);
        // now decrypt
        token = crate::auto_encrypt_decrypt_with_ssh_mod::decrypt_with_ssh_from_file(&crates_io_token_path_expanded);
    }
    // return
    token
}