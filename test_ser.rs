use openai_oxide::types::responses::*;

fn main() {
    let req = ResponseCreateRequest::new("gpt-5.4")
        .input("What is the capital of France? One word.")
        .max_output_tokens(16);
    println!("{}", serde_json::to_string(&req).unwrap());
}
