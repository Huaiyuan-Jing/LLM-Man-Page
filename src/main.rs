use crate::encrypt::{load_config, save_config};
use crate::llm::gen_man_page;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
mod encrypt;
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
    /// Reset all or specific command buffer, e.g. 'llman --reset' to reset all buffer, 'llman --reset cat' to reset buffer of cat command
    #[arg(long)]
    reset: Option<Option<String>>,
    /// Command you want to check
    man: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmConfig {
    pub engine: String,                                   // "openai" or "ollama"
    pub model: String,              // model name, e.g. "gpt-4-turbo", "llama3"
    pub openai_key: Option<String>, // OpenAI API key, if using OpenAI
    pub gemini_key: Option<String>, // Gemini API key, if using Gemini
    pub buffer: HashMap<String, HashMap<String, String>>, // store past generation result
}
impl LlmConfig {
    pub fn reset_all_buffer(&mut self) {
        self.buffer.clear();
    }
    pub fn reset_buffer_key(&mut self, key: &str) {
        for (_, dict) in &mut self.buffer {
            dict.remove(key);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let mut cfg = load_config().unwrap_or_else(|_| LlmConfig {
        engine: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
        openai_key: None,
        gemini_key: None,
        buffer: HashMap::new(),
    });
    let args = Args::parse();
    if let Some(reset) = args.reset {
        match reset {
            None => {
                cfg.reset_all_buffer();
                println!("All buffers cleared.");
            }
            Some(key) => {
                cfg.reset_buffer_key(&key);
            }
        }
    }
    let key = args.key;
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
    if let Some(man_cmd) = args.man {
        println!(
            "{}",
            gen_man_page(&mut cfg, &man_cmd, args.custom_prompt)
                .await
                .unwrap()
        );
    }
    save_config(&cfg).unwrap();
    Ok(())
}
