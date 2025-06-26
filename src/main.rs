use clap::Parser;
use home;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs;
use std::io::{self};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "llm-man-page",
    version = "0.1.0",
    about = "Better man page supported by LLM",
    arg_required_else_help = true
)]
struct Args {
    /// Prompt you want to send to GPT
    #[arg(short, long)]
    prompt: Option<String>,
    /// Save or Update OpenAI API key
    #[arg(long)]
    key: Option<String>,
    /// Command you want to check
    #[arg(short, long)]
    man: Option<String>,
}

fn get_gpt_response(prompt: &String) -> String {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = ChatBody {
        model: "gpt-4.1".to_string(),
        max_tokens: None,
        temperature: Some(0.2_f32),
        top_p: Some(0.2_f32),
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

fn fetch_man_page(cmd: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://man7.org/linux/man-pages/man1/{}.1.html", cmd);
    let res = reqwest::blocking::get(url)?.text()?;

    let document = Html::parse_document(&res);

    // man7.org 的 man page 通常包在 <pre> 标签内
    let sel = Selector::parse("pre").unwrap();
    if let Some(element) = document.select(&sel).next() {
        let man_text = element.text().collect::<Vec<_>>().join("\n");
        Ok(man_text)
    } else {
        Err("Cannot get man info".into())
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if let Some(key) = args.key {
        save_key(&key).expect("Fail to save key");
        println!("OPENAI_API_KEY saved");
        return Ok(());
    }
    let key = load_key().unwrap_or_else(|| {
        eprintln!("No valid OPENAI_API_KEY detected");
        std::process::exit(1);
    });
    unsafe { std::env::set_var("OPENAI_API_KEY", key) };
    if let Some(prompt) = args.prompt {
        println!("{}", get_gpt_response(&prompt));
        return Ok(());
    }
    if let Some(man_cmd) = args.man {
        let raw = fetch_man_page(&man_cmd).expect("Fail to get man page info");
        let reformatted = get_gpt_response(&format!(
            "Here is the man page for `{}`:\n{}\n\nrewrite it to generate a more readable and clear version, and get the full rewrite of this entire man page. And use plain text instead of markdown format",
            man_cmd, raw
        ));
        println!("{}", reformatted);
        return Ok(());
    }
    Ok(())
}
