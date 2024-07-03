use std::io::{BufRead, BufReader};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use serialport;

#[derive(Debug)]
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

fn main() {
    let (tx, rx) = channel::<Message>();

    thread::spawn(move || {
        let port = serialport::new("/dev/ttyACM0", 115_200)
            .timeout(Duration::from_secs(60 * 60))
            .open()
            .expect("ttyACM0");
        let mut reader = BufReader::new(port);
        let mut line = String::new();
        let mut messages : HashMap<String, String> = HashMap::new();

        loop {
            let _ = match reader.read_line(&mut line) {
                Ok(_) => {
                    let mut msg = Message::from(&line);
                    if msg.message.ends_with("$") {
                        let to_insert = msg.message[..msg.message.len() - 1].to_string();
                        messages.entry(msg.author)
                            .and_modify(|message| message.push_str(&to_insert))
                            .or_insert(to_insert);
                    } else {
                        let prev_msg = messages.remove(&msg.author).unwrap_or("".to_string());
                        msg.message = prev_msg + &msg.message;
                        if let Err(e) = tx.send(msg) {
                            dbg!(e);
                        }
                    }

                    line = String::new();
                },
                Err(_) => break,
            };
        }
    });

    for msg in rx {
        println!("{}: {}", msg.author, msg.message);
    }
}
