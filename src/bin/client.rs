use std::net::TcpStream;
use std::io::{ BufRead, BufReader, Write, stdin };
use std::thread::{ spawn };

const SERVER_ADDR: &str = "gremeline.com:8000";

fn main() {
    println!("Enter your name: ");

    let mut name = String::new();
    stdin().read_line(&mut name)
        .expect("Failed to read name");

    let name = name.trim();

    let mut read_socket = TcpStream::connect(SERVER_ADDR)
            .expect("Failed to connect to server");

    println!("Connected to server, you can start sending messages !");

    writeln!(read_socket, "{}", name)
        .expect("Failed to send name to server");

    {
        // Spawn a thread with a reference to the socket
        let write_socket = read_socket.try_clone()
            .expect("Failed to clone socket");
        spawn(move || message_sender(write_socket));
    }

    let mut reader = BufReader::new(read_socket);

    loop {
        let mut message = String::new();
        reader.read_line(&mut message)
            .expect("Failed to receive message");
        if message.len() == 0 {
            println!("Disconnected from server");
            break;
        }

        let message = message.trim();

        if message.len() > 0 {
            println!("{}", message);
        }
    }
}

fn message_sender(mut write_socket: TcpStream) {
    loop {
        let mut message = String::new();
        stdin().read_line(&mut message)
            .expect("Failed to read message");

        writeln!(write_socket, "{}", message.trim())
            .expect("Failed to send message to server");
    }
}