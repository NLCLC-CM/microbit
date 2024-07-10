use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{SendError, channel};
use std::thread;
use std::time::Duration;

use serialport;
use handlebars::Handlebars;
use warp::Filter;
use warp::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    author: String,
    message: String,
}

impl From<&String> for Message {
    /// # Example
    /// `<author>,<message>`
    fn from(line: &String) -> Self {
        if let Some(comma_index) = line.find(",") {
            Message {
                author: line[..comma_index].to_string(),
                message: line[comma_index + 1..].trim_end().to_string(),
            }
        } else {
            Message {
                author: "Unknown author".to_string(),
                message: line.trim_end().to_string(),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let all_messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(vec![
        Message {
            author: String::from("Person 1"),
            message: String::from("This is a sample message"),
        },
        Message {
            author: String::from("Person 2"),
            message: String::from("This is another sample message"),
        },
        Message {
            author: String::from("Person 3"),
            message: String::from("This is the third sample message"),
        },
    ]));

    {
        let all_messages = Arc::clone(&all_messages);
        thread::spawn(move || {
            let port = serialport::new("/dev/ttyACM0", 115_200)
                .timeout(Duration::from_secs(60 * 60))
                .open()
                .expect("ttyACM0");
            let mut reader = BufReader::new(port);
            let mut line = String::new();
            let mut messages: HashMap<String, String> = HashMap::new();

            loop {
                let _ = match reader.read_line(&mut line) {
                    Ok(_) => {
                        let mut msg = Message::from(&line);
                        if msg.message.ends_with("$") {
                            let to_insert = msg.message[..msg.message.len() - 1].to_string();
                            messages
                                .entry(msg.author)
                                .and_modify(|message| message.push_str(&to_insert))
                                .or_insert(to_insert);
                            } else {
                                let prev_msg = messages.remove(&msg.author).unwrap_or("".to_string());
                                msg.message = prev_msg + &msg.message;
                                if let Ok(mut all_messages) = all_messages.lock() {
                                    all_messages.push(msg.clone());
                                }
                        }

                        line = String::new();
                    }
                    Err(_) => break,
                };
            }
        });
    }

    let (tx, rx) = channel();
    {
        let all_messages = Arc::clone(&all_messages);
        thread::spawn(move || {
            for msg in rx {
                if let Ok(mut all_messages) = all_messages.lock() {
                    all_messages.push(msg);
                }
            }
        });
    }

    let mut hb = Handlebars::new();
    hb.register_template_file("index", "public/messaging.hbs").unwrap();
    hb.register_template_file("tmpl/messages", "public/templates/messages.hbs").unwrap();
    // this causes actual pain
    let hb1 = Arc::new(hb);
    let hb2 = Arc::clone(&hb1);

    let index = warp::get()
        .and(warp::path::end())
        .map(move || hb1.render("index", &json!({})).unwrap_or_else(|e| e.to_string()))
        .map(warp::reply::html);

    let messaging = warp::get()
        .and(warp::path("messages"))
        .map(move || hb2.render("tmpl/messages", &json!({"messages": *all_messages})).unwrap_or_else(|e| e.to_string()))
        .map(warp::reply::html);

    let message = warp::post()
        .and(warp::path("message"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::form())
        .map(move |msg: Message| tx.send(msg))
        .map(|result: Result<(), SendError<_>>| if result.is_err() {
            warp::reply::with_status("error", StatusCode::BAD_REQUEST)
        } else {
            warp::reply::with_status("
      <input type=\"text\" name=\"author\" placeholder=\"Author\" />
      <input type=\"text\" name=\"message\" placeholder=\"Type your message here!\" />
      <input type=\"submit\" />", StatusCode::OK)
        });

    warp::serve(index.or(messaging).or(message))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
