use std::net::{ TcpListener, TcpStream };
use std::thread::{ spawn, sleep };
use std::sync::mpsc::{ Receiver, Sender, channel };
use std::time::Duration;

const SERVER_ADDR: &str = "0.0.0.0:8000";

enum Action {
    Join(String),
    Send(Message),
    Quit(String),
}

struct Message {
    from: String,
    text: String,
}

impl Message {
    fn new(from: &str, text: &str) -> Message {
        Message {
            from: from.into(),
            text: text.into(),
        }
    }
}

fn main() {
    let listener = TcpListener::bind(SERVER_ADDR)
        .expect("Failed to bind server address");

    let (channel_sender, receiver) = channel();

    spawn(move || dispatcher(receiver));

    for socket in listener.incoming() {
        match socket {
            Ok(socket) => {
                let (sender, receiver) = channel();
                let read_socket = socket.try_clone()
                    .expect("Failed to clone socket");

                spawn(move || handle_client(read_socket, sender));

                channel_sender.send((receiver, socket))
                    .expect("Failed to transfer receiver to dispatch thread");
            },
            Err(e) => {
                eprintln!("An error occurs while reading socket: {:?}", e);
            }
        }
    }
}

fn handle_client(read_socket: TcpStream, sender: Sender<Action>) {
    use std::io::{ BufRead, BufReader };
    use Action::*;

    let mut reader = BufReader::new(read_socket);

    let mut name = String::new();
    reader.read_line(&mut name)
        .expect("Failed to receive name");

    let name = name.trim();

    sender.send(Join(name.into()))
        .expect("Failed to transfer greetings to main thread");

    loop {
        let mut message = String::new();
        if let Err(e) = reader.read_line(&mut message) {
            eprintln!("an error occurred while reading message from {}: {:?}", name, e);
            break;
        } else if message.len() == 0 {
            // Socket is closed, message is empty.
            break;
        }

        let message = message.trim();
        if message.len() > 0 {
            sender.send(Send(Message::new(name, message)))
                .expect("Failed to transfer message to main thread");
        }
    }
    sender.send(Quit(name.into()))
        .expect("Failed to transfer quit info to main thread");
}

fn dispatcher(channel_receiver: Receiver<(Receiver<Action>, TcpStream)>) {
    use Action::*;
    use std::io::Write;

    let mut receivers = Vec::new();

    loop {
        // Try to get a new receiver from main thread.
        if let Ok(receiver) = channel_receiver.try_recv() {
            receivers.push(receiver);
        }

        // Handle messages and filter receivers
        let mut messages = Vec::new();
        receivers = receivers.into_iter().filter(|&(ref receiver, _)| {
            if let Ok(action) = receiver.try_recv() {
                match action {
                    Join(name) => {
                        messages.push(format!("{0} joined the chat, Hi {0}!", name));
                        true
                    },
                    Send(msg) => {
                        messages.push(format!("{}: {}", msg.from, msg.text));
                        true
                    },
                    Quit(name) => {
                        messages.push(format!("{0} quitted the chat, bye {0}!", name));
                        false
                    },
                }
            } else {
                true
            }
        }).collect();

        // Send messages
        for msg in messages {
            for &mut (_, ref mut socket) in receivers.iter_mut() {
                if let Err(e) = writeln!(socket, "{}", msg) {
                    eprintln!("Failed to send message : {:?}", e);
                }
            }
        }

        sleep(Duration::from_millis(1));
    }
}