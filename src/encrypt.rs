use home;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmConfig {
    pub engine: String,             // "openai" or "ollama"
    pub model: String,              // model name, e.g. "gpt-4-turbo", "llama3"
    pub openai_key: Option<String>, // OpenAI API key, if using OpenAI
    pub gemini_key: Option<String>, // Gemini API key, if using Gemini
}

pub fn get_config_file() -> PathBuf {
    let mut path = get_app_folder();
    path.push(".llm_man_page_config.json");
    path
}

pub fn save_config(cfg: &LlmConfig) -> io::Result<()> {
    let path = get_config_file();
    let plain_text = serde_json::to_string_pretty(cfg).unwrap();
    let encrypted_str = encrypt_config_str(plain_text);
    fs::write(&path, encrypted_str)?;
    Ok(())
}

pub fn load_config() -> Option<LlmConfig> {
    let path = get_config_file();
    let encrypted_str = fs::read_to_string(path).ok()?;
    let decrypted_str = decrypt_config_str(encrypted_str);
    serde_json::from_str(&decrypted_str).ok()
}

pub fn get_app_folder() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".llman");
    path
}

pub fn make_folder(path: PathBuf) -> io::Result<()> {
    if !path.exists() {
        match fs::create_dir(path) {
            Ok(()) => return Ok(()),
            Err(e) => return Err(e),
        }
    }

    // file exist, test if it's a dir
    if path.is_dir() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "A common file with the same name already exists.",
        ))
    }
}

pub fn get_gpg_folder() -> PathBuf {
    let mut path = get_app_folder();
    path.push("gpgkey");
    path
}

pub fn gpg_key_exist() -> bool {
    // get gpg dir
    let gpg_home = get_gpg_folder();

    let status_code = Command::new("gpg")
        .arg("--homedir")
        .arg(gpg_home.to_str().unwrap())
        .arg("--list-secret-keys")
        .arg("llman@nonexist.com")
        .status()
        .expect("Failed to invoke gpg --list-secret-keys");

    status_code.success()
}

pub fn gen_gpg_key() -> io::Result<()> {
    // make config file
    let conf = format!(
        r#"
%echo Generating ECC GPG key
Key-Type: eddsa
Key-Curve: ed25519
Subkey-Type: ecdh
Subkey-Curve: cv25519
Name-Real: llman
Name-Email: llman@nonexist.com
Expire-Date: 0
%commit
%echo Done
"#,
    );

    let mut temp = std::env::temp_dir();
    temp.push("gpg-conf");
    fs::write(&temp, conf).unwrap();

    // use customized gpg home dir
    let gpg_home = get_gpg_folder();

    let exit_status = Command::new("gpg")
        .arg("--homedir")
        .arg(gpg_home.to_str().unwrap())
        .arg("--gen-key")
        .arg("--batch")
        .arg(temp.to_str().unwrap())
        .env("DISPLAY", "")
        .status()
        .expect("Failed to run gpg key generator.");

    if !exit_status.success() {
        panic!("Failed to generate gpg key pairs.");
    } else {
        Ok(())
    }
}

pub fn decrypt_config_str(s: String) -> String {
    let gpg_home = get_gpg_folder();
    let args = ["--homedir", gpg_home.to_str().unwrap(), "--decrypt"];
    xcrypt_config_str(s, &args)
}

pub fn encrypt_config_str(s: String) -> String {
    let gpg_home = get_gpg_folder();
    let args = [
        "--homedir",
        gpg_home.to_str().unwrap(),
        "--armor",
        "--recipient",
        "llman@nonexist.com",
        "--encrypt",
    ];
    xcrypt_config_str(s, &args)
}

pub fn xcrypt_config_str(s: String, args: &[&str]) -> String {
    let mut gpg = Command::new("gpg")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .env("DISPLAY", "")
        .spawn()
        .expect("Failed to start gpg encrypt / decrypt command");

    {
        let stdin = gpg
            .stdin
            .as_mut()
            .expect("Failed to open stdin of gpg encrypt / decrypt command");
        stdin
            .write_all(s.as_bytes())
            .expect("Failed to write input to stdin of gpg encrypt / decrypt command");
    }

    let output = gpg.wait_with_output().expect("Failed to read gpg output.");

    if !output.status.success() {
        panic!(
            "gpg encrypt / decrypt operation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
