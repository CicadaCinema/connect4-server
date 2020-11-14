use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

fn handle_two_clients(mut stream1: TcpStream, mut stream2: TcpStream) {
    let mut data = [0 as u8; 1]; // using 1 byte buffer

    stream1.write(b"1").unwrap();
    stream2.write(b"2").unwrap();

    loop {
        match stream1.read(&mut data) {
            Ok(size) => {
                stream2.write(&data[0..size]).unwrap();
                data = [0 as u8; 1];
            },
            Err(_) => {
                println!("An error occurred, terminating connection with {}", stream1.peer_addr().unwrap());
                stream1.shutdown(Shutdown::Both).unwrap();
            }
        }
        match stream2.read(&mut data) {
            Ok(size) => {
                stream1.write(&data[0..size]).unwrap();
                data = [0 as u8; 1];
            },
            Err(_) => {
                println!("An error occurred, terminating connection with {}", stream1.peer_addr().unwrap());
                stream2.shutdown(Shutdown::Both).unwrap();
            }
        }
    }
}

fn main() {
    let mut stream_vector: Vec<TcpStream> = vec![];

    let listener = TcpListener::bind("0.0.0.0:32032").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 32032");


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //println!("New connection: {}", stream.peer_addr().unwrap());
                println!("got another stream!");
                stream_vector.push(stream);
                /* thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                }); */
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }

        if stream_vector.len() == 2 {
            println!("got two streams!");
            // replace unwrap with something else
            let stream1 = stream_vector.pop().unwrap();
            let stream2 = stream_vector.pop().unwrap();
            handle_two_clients(stream1, stream2);
        }
    }
    // close the socket server
    drop(listener);
}