use openai_api_rust::chat::*;
use openai_api_rust::completions::*;
use openai_api_rust::*;

fn get_gpt_response() -> String{
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = ChatBody {
        model: "gpt-4.1".to_string(),
        max_tokens: Some(7),
        temperature: Some(0.5_f32),
        top_p: Some(0.5_f32),
        n: Some(2),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
        }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();
    message.content.clone()
}

fn main() {
    println!("{}", get_gpt_response());
}
