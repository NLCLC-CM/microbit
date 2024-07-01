use std::io::BufRead;
use std::sync::mpsc::channel;
use std::thread;

struct Message {
    author: String,
    message: String,
}

impl From<String> for Message {
    /// # Example
    /// `<author>,<message>`
    fn from(line: String) -> Self {
        if let Some(comma_index) = line.find(",") {
            Message {
                author: line[..comma_index].to_string(),
                message: line[comma_index + 1..].to_string(),
            }
        } else {
            Message {
                author: "Unknown author".to_string(),
                message: line,
            }
        }
    }
}

fn main() {
    let (tx, rx) = channel::<Message>();

    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();

        loop {
            let _ = match handle.read_line(&mut line) {
                Ok(_) => {
                    let _ = tx.send(line.clone().into());
                },
                Err(_) => break,
            };
        }
    });

    for msg in rx {
        println!("{}: {}", msg.author, msg.message);
    }
}
