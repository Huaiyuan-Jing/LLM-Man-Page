use clap::Parser;
use home;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod llm;

#[derive(Parser, Debug)]
#[command(
    name = "llm-man-page",
    version = "0.1.0",
    about = "Better man page supported by LLM",
    arg_required_else_help = true
)]
struct Args {
    /// Save or Update OpenAI API key
    #[arg(long)]
    key: Option<String>,
    /// Set LLM engine: 'openai', 'ollama' or 'google'
    #[arg(long)]
    engine: Option<String>,
    /// Set model name for LLM, e.g. 'gpt-4-turbo' or 'llama3'
    #[arg(long)]
    model: Option<String>,
    /// Extra info
    #[arg(short, long)]
    custom_prompt: Option<String>,
    /// Command you want to check
    man: Option<String>,
}

fn fetch_man_page(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let man_output = Command::new("man").arg(cmd).output()?;
    if !man_output.status.success() {
        return Err(format!("Failed to get man page for {}", cmd).into());
    }
    Ok(String::from_utf8_lossy(&man_output.stdout).to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LlmConfig {
    engine: String,             // "openai" or "ollama"
    model: String,              // model name, e.g. "gpt-4-turbo", "llama3"
    openai_key: Option<String>, // OpenAI API key, if using OpenAI
    gemini_key: Option<String>, // Gemini API key, if using Gemini
}

fn get_config_file() -> PathBuf {
    let mut path = get_app_folder();
    path.push(".llm_man_page_config.json");
    path
}

fn save_config(cfg: &LlmConfig) -> io::Result<()> {
    let path = get_config_file();
    let plain_text = serde_json::to_string_pretty(cfg).unwrap();
    let encrypted_str = encrypt_config_str(plain_text);
    fs::write(&path, encrypted_str)?;
    Ok(())
}

fn load_config() -> Option<LlmConfig> {
    let path = get_config_file();
    let encrypted_str = fs::read_to_string(path).ok()?;
    let decrypted_str = decrypt_config_str(encrypted_str);
    serde_json::from_str(&decrypted_str).ok()
}

fn get_app_folder() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".llman");
    path
}

fn make_folder(path: PathBuf) -> io::Result<()> {
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

fn get_gpg_folder() -> PathBuf {
    let mut path = get_app_folder();
    path.push("gpgkey");
    path
}

fn gpg_key_exist() -> bool {
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

fn gen_gpg_key() -> io::Result<()> {
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

fn decrypt_config_str(s: String) -> String {
    let gpg_home = get_gpg_folder();
    let args = ["--homedir", gpg_home.to_str().unwrap(), "--decrypt"];
    xcrypt_config_str(s, &args)
}

fn encrypt_config_str(s: String) -> String {
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

fn xcrypt_config_str(s: String, args: &[&str]) -> String {
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

#[tokio::main]
async fn main() -> Result<(), ()> {
    // 1. make app folder
    let app_folder_path = get_app_folder();
    make_folder(app_folder_path).unwrap();

    // 2. make gpg folder second
    let gpg_folder_path = get_gpg_folder();
    make_folder(gpg_folder_path).unwrap();

    // 3. create gpg key to protect api keys in config file
    if !gpg_key_exist() {
        gen_gpg_key().unwrap();
    }

    // 4. start business logic
    let args = Args::parse();
    let key = args.key;
    let mut cfg = load_config().unwrap_or_else(|| LlmConfig {
        engine: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
        openai_key: None,
        gemini_key: None,
    });
    if let Some(engine) = args.engine {
        cfg.engine = engine.trim().to_lowercase();
    }
    if let Some(model) = args.model {
        cfg.model = model.trim().to_lowercase();
    }
    if let Some(key) = key {
        if cfg.engine == "openai" {
            cfg.openai_key = Some(key.clone());
        } else if cfg.engine == "google" {
            cfg.gemini_key = Some(key.clone());
        }
    }

    let _ = save_config(&cfg);

    if let Some(man_cmd) = args.man {
        if cfg.engine == "openai" {
            unsafe {
                std::env::set_var(
                    "OPENAI_API_KEY",
                    match cfg.openai_key {
                        Some(ref key) => key.clone(),
                        None => {
                            println!(
                                "OpenAI API key is not set. Please set it using --key option or in the config file."
                            );
                            return Err(());
                        }
                    },
                )
            };
        } else if cfg.engine == "google" {
            unsafe {
                std::env::set_var(
                    "GEMINI_API_KEY",
                    match cfg.gemini_key {
                        Some(ref key) => key.clone(),
                        None => {
                            println!(
                                "Gemini API key is not set. Please set it using --key option or in the config file."
                            );
                            return Err(());
                        }
                    },
                )
            };
        }
        let raw = fetch_man_page(&man_cmd).expect("Fail to get man page info");
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Generating improved man page…");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
        let prompt = match args.custom_prompt {
            None => format!(
                "Here is the man page for {}: [{}]\n
                1. rewrite the explanation part of each command, remember don't change any other content.\n
                2. add example of usage after explaination of commands.\n
                3. double check to make sure you contains all the commands and explain them correctly.\n
                If you are not sure about file content or codebase structure pertaining to the user's request, use your tools to read files and gather the relevant information: do NOT guess or make up an answer.\n
                Directly return the content without any other useless information. Do not include any additional text after your response.\n",
                    man_cmd, raw
            ),
            Some(prompt) => format!(
                "Here is the man page for {}: [{}]\n
                Use the previous man page to solve the following task: {}\n
                If you are not sure about file content or codebase structure pertaining to the user's request, use your tools to read files and gather the relevant information: do NOT guess or make up an answer.\n
                Explain all the options and arguments used in your answer by referencing the related man page content.\n
                Do not include any markdown format in response.\n
                Directly return the content without any other useless information. Do not include any additional text after your response.\n",
                man_cmd, raw, prompt
            ),
        };
        let reformatted = match cfg.engine.as_str() {
            "ollama" => llm::get_ollama_response(&prompt, &cfg.model).await,
            "openai" => llm::get_gpt_response(&prompt, &cfg.model),
            "google" => llm::get_google_response(&prompt).await,
            _ => {
                spinner.finish_and_clear();
                println!("Unsupported engine: {}", cfg.engine);
                return Err(());
            }
        };
        spinner.finish_and_clear();
        println!("{}", reformatted);
        return Ok(());
    }
    Ok(())
}
