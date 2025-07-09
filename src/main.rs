use clap::Parser;
use gemini_rust::Gemini;
use home;
use indicatif::{ProgressBar, ProgressStyle};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
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
    /// Set LLM engine: 'openai', 'ollama' or 'google'
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
        messages: vec![openai_api_rust::Message {
            role: openai_api_rust::Role::User,
            content: prompt.clone(),
        }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();
    message.content.clone()
}

async fn get_google_response(prompt: &String) -> String {
    let client = Gemini::new(&std::env::var("GEMINI_API_KEY").unwrap());

    let response = client
        .generate_content()
        .with_system_prompt("You are a helpful assistant.")
        .with_user_message(prompt.clone())
        .execute()
        .await
        .unwrap();
    response.text()
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
    let s = serde_json::to_string_pretty(cfg).unwrap();
    fs::write(&path, s)?;
    Ok(())
}

fn load_config() -> Option<LlmConfig> {
    let path = get_config_file();
    let s = fs::read_to_string(path).ok()?;
    serde_json::from_str(&s).ok()
}

fn get_app_folder() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".llman");
    path
}

fn make_app_folder() -> io::Result<()> {
    let path = get_app_folder();
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
        Err(io::Error::new(io::ErrorKind::AlreadyExists, "A common file with the same name already exists."))
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    // make app folder first
    make_app_folder().unwrap();

    // then parse cmdline args
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
            "openai" => get_gpt_response(&prompt, &cfg.model),
            "google" => get_google_response(&prompt).await,
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
