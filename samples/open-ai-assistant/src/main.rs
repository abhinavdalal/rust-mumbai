use dotenv::dotenv;
use std::env::var;
use std::io;

use openai_api_rs::v1::{
    api::Client, message::CreateMessageRequest, message::MessageRole, run::CreateRunRequest,
    thread::CreateThreadRequest,
};

// static OPENAI_API_BASE: &str = "https://api.openai.com/v1";

fn main() {
    dotenv().ok();
    let openai_api_key: String = var("OPENAI_API_KEY").expect("open api key not found in env");
    let client = Client::new(openai_api_key);
    let thread = client
        .create_thread(CreateThreadRequest::new())
        .expect("Unable to create a thread");
    println!("How can I help you today?");

    let mut i = 0;

    loop {
        println!("iteration: {}", i);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Input parse error!");
        // println!("{}", input);
        if input == "bye\n" || input == "quit\n" {
            break;
        }
        // messages.push(input.as_str());

        client
            .create_message(
                thread.id.to_string(),
                CreateMessageRequest::new(MessageRole::user, input.to_string()),
            )
            .expect("Unable to create a message");

        let run = client
            .create_run(
                thread.id.to_string(),
                CreateRunRequest::new(
                    var("OPENAI_ASSISTANT_ID").expect("assistants key not found in env"),
                )
                .instructions(
                    "Address the user as Abhinav. The user has a premium account.".to_string(),
                ),
            )
            .expect("Unable to create a run");

        loop {
            let run_result = client
                .retrieve_run(thread.id.clone(), run.id.clone())
                .unwrap();
            if run_result.status == "completed" {
                break;
            } else {
                println!("waiting...");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        let list_message_result = client.list_messages(thread.id.clone()).unwrap();
        println!(
            "{:?}: {:?}",
            list_message_result.data[0].role, list_message_result.data[0].content[0].text.value
        );
        i += 1;
    }
}
