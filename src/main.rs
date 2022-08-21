use std::io::{self, ErrorKind, Read, Write};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

mod constants {
    pub const LOCAL_HOST: &str = "127.0.0.1:6000";
    pub const MAX_MSG_LENGTH: usize = 32;
    pub const DEFAULT_THREAD_SLEEP_MILLIS: u64 = 100;
}

mod  utils {

    pub fn sleep_for(millis: u64) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }
}

mod client {
    use std::net::TcpStream;

    use super::constants;

    pub fn get_new_tcp_stream() -> TcpStream {
        let client = TcpStream::connect(constants::LOCAL_HOST)
            .expect("TCP stream failed to connect to the server address.");

        client.set_nonblocking(true)
            .expect("Failed to set the non-blocking flag.");

        return client;
    }
}

fn main() {
    let mut client = client::get_new_tcp_stream();

    let (
        sender,
        receiver
    ) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buffer = vec![0; constants::MAX_MSG_LENGTH];
        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let message = buffer.into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();

                let message = String::from_utf8(message)
                    .expect("Invalid format, cannot be parsed as UTF-8");

                println!("message received: {:?}", message);
            },
            Err(ref error) if error.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                eprintln!("The connection with server was severed.");
                break;
            }
        }

        match receiver.try_recv() {
            Ok(message) => {
                let mut buffer = message.clone().into_bytes();
                buffer.resize(constants::MAX_MSG_LENGTH, 0);

                client.write_all(&buffer)
                    .expect("Writing to the TCP stream has failed.");

                println!("sent the message: {:?}", message);
            },
            Err(TryRecvError::Empty) => {
                // no need for handling
            },
            Err(TryRecvError::Disconnected) => break,
        }

        utils::sleep_for(constants::DEFAULT_THREAD_SLEEP_MILLIS)
    });

    println!("Write a message:");

    loop {
        let mut buffer = String::new();

        io::stdin().read_line(&mut buffer)
            .expect("Reading from stdin has failed.");

        let message = buffer.trim().to_string();
        if message == ":quit" || sender.send(message).is_err() {
            break;
        }
    }

    println!("bye bye!");

}