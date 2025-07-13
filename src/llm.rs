use crate::LlmConfig;
use gemini_rust::Gemini;
use indicatif::{ProgressBar, ProgressStyle};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use std::collections::HashMap;
use std::process::Command;

fn setup_key(cfg: &LlmConfig) -> Result<(), &str> {
    if cfg.engine == "openai" {
        unsafe {
            std::env::set_var(
                "OPENAI_API_KEY",
                match cfg.openai_key {
                    Some(ref key) => key.clone(),
                    None => {
                        return Err("OpenAI API key is not set. Please set it using --key option.");
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
                        return Err("Gemini API key is not set. Please set it using --key option");
                    }
                },
            )
        };
    }
    Ok(())
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

fn fetch_man_page(cmd: &str) -> Result<String, String> {
    let man_output = Command::new("man").arg(cmd).output().unwrap();
    if !man_output.status.success() {
        return Err(format!("Failed to get man page for {}", cmd));
    }
    Ok(String::from_utf8_lossy(&man_output.stdout).to_string())
}

pub async fn gen_man_page(
    cfg: &mut LlmConfig,
    man_cmd: &String,
    custom_prompt: Option<String>,
) -> Result<String, String> {
    setup_key(&cfg).unwrap();
    if !cfg.buffer.contains_key(&cfg.model) {
        cfg.buffer.insert(cfg.model.clone(), HashMap::new());
    } else if cfg.buffer.get(&cfg.model).unwrap().contains_key(man_cmd) && custom_prompt.is_none() {
        return Ok(cfg
            .buffer
            .get(&cfg.model)
            .unwrap()
            .get(man_cmd)
            .unwrap()
            .clone());
    }
    let raw = fetch_man_page(&man_cmd).expect("Fail to get man page info");
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Generating improved man page…");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
    let prompt = match custom_prompt.clone() {
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
        "ollama" => get_ollama_response(&prompt, &cfg.model).await,
        "openai" => get_gpt_response(&prompt, &cfg.model),
        "google" => get_google_response(&prompt).await,
        _ => {
            spinner.finish_and_clear();
            return Err(format!("Unsupported engine: {}", cfg.engine));
        }
    };
    spinner.finish_and_clear();
    if custom_prompt.is_none() {
        cfg.buffer
            .get_mut(&cfg.model)
            .unwrap()
            .insert(man_cmd.clone(), reformatted.clone());
    }
    Ok(reformatted)
}
