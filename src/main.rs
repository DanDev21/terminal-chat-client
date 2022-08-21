use std::io::{Read, ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, TryRecvError};
use std::{io, thread};
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:6000";
const MAX_MSG_LENGTH: u8 = 32;
const THREAD_SLEEP_TIME: u64 = 100;
const QUIT_COMMAND: &str = ":quit";

fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis))
}

fn main() {
    let mut tcp_stream = TcpStream::connect(SERVER_ADDR)
        .expect("Client failed to connect to the server.");

    tcp_stream.set_nonblocking(true)
        .expect("Failed to set client's non-blocking flag.");

    let (
        sender,
        receiver,
    ) = channel::<String>();

    // message receiving thread
    thread::spawn(move || {
        loop {
            let mut buffer = vec![0, MAX_MSG_LENGTH];

            // trying to read the messages coming from the server
            match tcp_stream.read_exact(&mut buffer) {
                Ok(_) => {
                    let message = buffer.into_iter()
                        .take_while(|&x| 0 != x)
                        .collect::<Vec<_>>();

                    println!("message received: {:?}", message);
                }
                Err(ref error) if ErrorKind::WouldBlock == error.kind() => {
                    continue; // this kind of error isn't important
                }
                Err(_) => {
                    eprintln!("connection to the server has been severed.");
                    break;
                }
            }

            // trying to receive messages
            match receiver.try_recv() {
                Ok(message) => {
                    let mut buffer = message.clone().into_bytes();
                    buffer.resize(MAX_MSG_LENGTH as usize, 0);

                    tcp_stream.write_all(&buffer)
                        .expect("Failed to write the message in to the socket.");

                    println!("message sent: {:?}", message);
                }
                Err(TryRecvError::Empty) => {
                    continue; // this case doesn't need to be handled
                }
                Err(TryRecvError::Disconnected) => {
                    eprintln!("you've been disconnected from the server.");
                    break;
                }
            }

            sleep(THREAD_SLEEP_TIME);
        }
    });

    println!("Write a message: ('{}' command if you want to exit the program)", QUIT_COMMAND);

    loop {
        let mut buffer = String::new();

        io::stdin().read_line(&mut buffer)
            .expect("Failed to read the input from the user.");

        let message = buffer.trim().to_string();

        if QUIT_COMMAND == message {
            println!("quiting...");
            break;
        }

        if sender.send(message.clone()).is_err() {
            eprintln!("Failed to send the message.");
            break;
        }

        println!("the message:\n{:?}\nhas been sent", message);
    }

    println!("bye!");
}
