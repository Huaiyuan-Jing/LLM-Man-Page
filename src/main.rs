use clap::Parser;
use home;
use indicatif::{ProgressBar, ProgressStyle};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

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
    /// Set LLM engine: 'openai' or 'ollama'
    #[arg(long)]
    engine: Option<String>,
    /// Set model name for LLM, e.g. 'gpt-4-turbo' or 'llama3'
    #[arg(long)]
    model: Option<String>,
    /// Command you want to check
    man: Option<String>,
}

async fn get_ollama_response(prompt: &String, model: &String) -> String {
    let ollama = Ollama::default();
    let res = ollama.generate(GenerationRequest::new(model.clone(), prompt));
    res.await.unwrap().response
}

fn get_gpt_response(prompt: &String, model: &String) -> String {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = ChatBody {
        model: model.clone(),
        max_tokens: None,
        temperature: Some(0.2_f32),
        top_p: Some(0.1_f32),
        n: Some(2),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message {
            role: Role::User,
            content: prompt.clone(),
        }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();
    message.content.clone()
}

fn get_key_file() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".openai_api_key");
    path
}

fn save_key(key: &str) -> io::Result<()> {
    let path = get_key_file();
    fs::write(&path, key)?;
    let mut perms = fs::metadata(&path)?.permissions();
    #[cfg(unix)]
    perms.set_mode(0o600);
    fs::set_permissions(&path, perms)?;
    Ok(())
}

fn load_key() -> Option<String> {
    if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        return Some(k);
    }
    let path = get_key_file();
    fs::read_to_string(path).ok()
}

fn fetch_man_page(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let man_output = Command::new("man").arg(cmd).output()?;
    if !man_output.status.success() {
        return Err(format!("Failed to get man page for {}", cmd).into());
    }

    let mut col = Command::new("col")
        .arg("-b")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = col.stdin.as_mut() {
        stdin.write(&man_output.stdout)?;
    } else {
        return Err("Failed to open stdin for col".into());
    }

    let output = col.wait_with_output()?;
    if !output.status.success() {
        return Err("col -b failed".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LlmConfig {
    engine: String, // "openai" or "ollama"
    model: String,  // model name, e.g. "gpt-4-turbo", "llama3"
}

fn get_config_file() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".llm_man_page_config.json");
    path
}

fn save_config(cfg: &LlmConfig) -> io::Result<()> {
    let path = get_config_file();
    let s = serde_json::to_string_pretty(cfg).unwrap();
    fs::write(&path, s)?;
    Ok(())
}

fn load_config() -> Option<LlmConfig> {
    let path = get_config_file();
    let s = fs::read_to_string(path).ok()?;
    serde_json::from_str(&s).ok()
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    if let Some(key) = args.key {
        save_key(&key).expect("Fail to save key");
        println!("OPENAI_API_KEY saved");
        return Ok(());
    }
    let mut cfg = load_config().unwrap_or_else(|| LlmConfig {
        engine: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
    });
    if let Some(engine) = args.engine {
        cfg.engine = engine;
    }
    if let Some(model) = args.model {
        cfg.model = model;
    }
    save_config(&cfg)?;
    let key = if cfg.engine == "openai" {
        load_key().unwrap_or_else(|| {
            eprintln!("No valid OPENAI_API_KEY detected");
            std::process::exit(1);
        })
    } else {
        String::new()
    };
    unsafe { std::env::set_var("OPENAI_API_KEY", key) };
    if let Some(man_cmd) = args.man {
        let raw = fetch_man_page(&man_cmd).expect("Fail to get man page info");
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Generating improved man page…");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
        let prompt = format!(
            "Here is the man page for {}: [{}]\n
            1. rewrite the explanation part of each command, remember don't change any other content.\n
            2. add example of usage after explaination of commands.\n
            3. double check to make sure you contains all the commands and explain them correctly.\n
            If you are not sure about file content or codebase structure pertaining to the user's request, use your tools to read files and gather the relevant information: do NOT guess or make up an answer.\n
            Directly return the content without any other useless information. Do not include any additional text after your response.\n",
                man_cmd, raw
            );
        let reformatted = match cfg.engine.as_str() {
            "ollama" => get_ollama_response(&prompt, &cfg.model).await,
            "openai" | _ => get_gpt_response(&prompt, &cfg.model),
        };
        spinner.finish_and_clear();
        println!("{}", reformatted);
        return Ok(());
    }
    Ok(())
}
