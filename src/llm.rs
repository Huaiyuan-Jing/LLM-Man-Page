use gemini_rust::Gemini;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use openai_api_rust::chat::*;
use openai_api_rust::*;

pub async fn get_ollama_response(prompt: &String, model: &String) -> String {
    let ollama = Ollama::default();
    let res = ollama.generate(GenerationRequest::new(model.clone(), prompt));
    res.await.unwrap().response
}

pub fn get_gpt_response(prompt: &String, model: &String) -> String {
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

pub async fn get_google_response(prompt: &String) -> String {
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
