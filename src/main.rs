use crate::encrypt::{LlmConfig, load_config, save_config};
use crate::llm::gen_man_page;
use clap::Parser;
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
    /// Command you want to check
    man: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let mut cfg = load_config().unwrap_or_else(|_| LlmConfig {
        engine: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
        openai_key: None,
        gemini_key: None,
    });
    let args = Args::parse();
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

    let _ = save_config(&cfg);

    if let Some(man_cmd) = args.man {
        println!(
            "{}",
            gen_man_page(&cfg, &man_cmd, args.custom_prompt)
                .await
                .unwrap()
        );
        return Ok(());
    }
    Ok(())
}
