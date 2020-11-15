use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::time::Duration;

fn handle_two_clients(mut stream1: TcpStream, mut stream2: TcpStream) {
    // using 1 byte buffer
    let mut data = [0 as u8; 1];

    // set timeouts
    // TODO: add error checking in case timeouts are not supported
    stream1.set_read_timeout(Some(Duration::new(60, 0))).unwrap();
    stream2.set_read_timeout(Some(Duration::new(60, 0))).unwrap();
    stream1.set_write_timeout(Some(Duration::new(20, 0))).unwrap();
    stream2.set_write_timeout(Some(Duration::new(20, 0))).unwrap();

    // let the clients know which one is 'player 1'
    stream1.write(b"1").unwrap();
    stream2.write(b"2").unwrap();

    // TODO: reduce number of lines by removing repeated code? this isn't a very high priority...
    loop {
        match stream1.read(&mut data) {
            Ok(size) => {
                let mut data_to_send: [u8; 3] = [0; 3];
                for i in 0..3 {
                    data_to_send[i] = 2;
                }

                // TODO: do some logic here before relaying command

                // pass the instructions to both clients
                stream1.write(&data_to_send).unwrap();
                stream2.write(&data_to_send).unwrap();

                // clear (received) data buffer
                data = [0 as u8; 1];
            },
            Err(_) => {
                //println!("An error occurred or the stream timed out, terminating connection with {}", stream1.peer_addr().unwrap());
                stream1.shutdown(Shutdown::Both).unwrap();
                stream2.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
        match stream2.read(&mut data) {
            Ok(size) => {
                let mut data_to_send: [u8; 3] = [0; 3];
                for i in 0..3 {
                    data_to_send[i] = 2;
                }

                // TODO: do some logic here before relaying command

                // pass the instructions to both clients
                stream1.write(&data_to_send).unwrap();
                stream2.write(&data_to_send).unwrap();

                // clear (received) data buffer
                data = [0 as u8; 1];
            },
            Err(_) => {
                //println!("An error occurred or the stream timed out, terminating connection with {}", stream2.peer_addr().unwrap());
                stream1.shutdown(Shutdown::Both).unwrap();
                stream2.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }
}

fn main() {
    let mut streams: Vec<TcpStream> = vec![];
    let listener = TcpListener::bind("0.0.0.0:32032").unwrap();
    //println!("Server listening on port 32032");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //println!("New connection: {}", stream.peer_addr().unwrap());
                streams.push(stream);
            }
            Err(e) => {
                // connection failed
                println!("Error: {}", e);
            }
        }

        if streams.len() == 2 {
            // remove the two latest streams from the vector and handle them in a new thread
            // unwrapping isn't a problem here because the length of the vector is exactly 2
            let stream2 = streams.pop().unwrap();
            let stream1 = streams.pop().unwrap();
            thread::spawn(|| {
                handle_two_clients(stream1, stream2);
            });
        }
    }
    // close the socket server
    drop(listener);
}