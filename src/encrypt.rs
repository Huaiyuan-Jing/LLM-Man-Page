use base64::{Engine, engine::general_purpose};
use chacha20poly1305::{
    Key, XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use home;
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{collections::HashMap, fs};

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

fn make_folder(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir(path.clone()).unwrap();
    }

    // file exist, test if it's a dir
    if path.is_dir() {
        Ok(())
    } else {
        Err(String::from("Path is not a folder"))
    }
}

fn get_app_folder() -> PathBuf {
    let mut path = home::home_dir().expect("Cannot access home dir");
    path.push(".llman");
    make_folder(&path).unwrap();
    path
}

fn get_config_path() -> PathBuf {
    let mut path = get_app_folder();
    path.push(".llman_config");
    path
}

/// Generate Random key and store into keyring
fn setup_key() -> Result<(), ()> {
    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    let entry = Entry::new("llman", "config_key").unwrap();
    entry
        .set_password(&general_purpose::STANDARD.encode(&key))
        .unwrap();
    Ok(())
}

/// Load key from keyring
fn load_key() -> Result<[u8; 32], ()> {
    let entry = Entry::new("llman", "config_key").unwrap();
    let b64 = entry.get_password().unwrap();
    let vec = general_purpose::STANDARD.decode(b64).unwrap();
    let mut key = [0u8; 32];
    key.copy_from_slice(&vec);
    Ok(key)
}

/// Use XChaCha20-Poly1305 Encrypt
fn encrypt_config(key: &[u8; 32], plaintext: &[u8]) -> Result<String, ()> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce = XNonce::default();
    rand::rng().fill_bytes(nonce.as_mut());
    let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
    let mut combined = nonce.as_slice().to_vec();
    combined.extend(ciphertext);
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(combined))
}

/// Use XChaCha20-Poly1305 Decrypt
fn decrypt_config(key: &[u8; 32], ciphertext_b64: &str) -> Result<Vec<u8>, ()> {
    let data = general_purpose::URL_SAFE_NO_PAD
        .decode(ciphertext_b64)
        .unwrap();
    let (nonce_bytes, ct) = data.split_at(24);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    Ok(cipher.decrypt(nonce_bytes.into(), ct).unwrap())
}

fn is_key_exist() -> bool {
    Entry::new("llman", "config_key")
        .unwrap()
        .get_password()
        .is_ok()
}

pub fn save_config(cfg: &LlmConfig) -> Result<(), ()> {
    if !is_key_exist() {
        setup_key().unwrap();
    }
    let key = load_key().unwrap();
    let new_json = serde_json::to_vec_pretty(&cfg).unwrap();
    let enc = encrypt_config(&key, &new_json).unwrap();
    std::fs::write(get_config_path(), enc).unwrap();
    Ok(())
}

pub fn load_config() -> Result<LlmConfig, ()> {
    if !is_key_exist() {
        setup_key().unwrap();
    }
    let key = load_key().unwrap();
    // load encrypt config file
    let encrypted = match std::fs::read_to_string(get_config_path()) {
        Ok(val) => val,
        Err(_) => return Err(()),
    };
    let json = decrypt_config(&key, &encrypted).unwrap();
    let cfg: LlmConfig = serde_json::from_slice(&json).unwrap();
    Ok(cfg)
}
