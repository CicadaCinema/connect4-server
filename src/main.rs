use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::time::Duration;

// sees if the instruction (which should already be applied to state) results in a game win for that player
fn check_win(state: &[[u8; 7]; 6], instruction: &[u8; 3]) -> bool {
    let player_id = &instruction[2];
    let mut count_4;

    // check horizontally
    count_4 = 0;
    for i in 0..7 {
        let cell_value = &state[instruction[0] as usize][i];
        if cell_value == player_id {
            count_4 += 1;
        } else {
            count_4 = 0;
        }

        if count_4 == 4{
            return true;
        }
    }

    // check vertically
    count_4 = 0;
    for i in 0..6 {
        let cell_value = &state[i][instruction[1] as usize];
        if cell_value == player_id {
            count_4 += 1;
        } else {
            count_4 = 0;
        }

        if count_4 == 4{
            return true;
        }
    }

    // check diagonally changing coords by (1, 1)
    count_4 = 0;
    for i in -3..4 {
        if (0..6).contains(&(instruction[0] as i32 + i)) && (0..7).contains(&(instruction[1] as i32 + i)) {
            let coord_0 = (instruction[0] as i32 + i) as usize;
            let coord_1 = (instruction[1] as i32 + i) as usize;

            let cell_value = &state[coord_0][coord_1];
            if cell_value == player_id {
                count_4 += 1;
            } else {
                count_4 = 0;
            }

            if count_4 == 4{
                return true;
            }
        }
    }

    // check diagonally changing coords by (1, -1)
    count_4 = 0;
    for i in -3..4 {
        if (0..6).contains(&(instruction[0] as i32 + i)) && (0..7).contains(&(instruction[1] as i32 - i)) {
            let coord_0 = (instruction[0] as i32 + i) as usize;
            let coord_1 = (instruction[1] as i32 - i) as usize;

            let cell_value = &state[coord_0][coord_1];
            if cell_value == player_id {
                count_4 += 1;
            } else {
                count_4 = 0;
            }

            if count_4 == 4{
                return true;
            }
        }
    }

    return false;
}

fn handle_two_clients(mut stream1: TcpStream, mut stream2: TcpStream) {
    let mut streams = [stream1, stream2];

    // using 1 byte buffer
    let mut received_data = [0 as u8; 1];

    // the height of each of the columns
    let mut free_cells_in_column = [6 as u8; 7];

    // the state of the game board
    let mut state = [[0; 7]; 6];

    for stream_id in 0..2 {
        // TODO: add error checking in case timeouts are not supported
        // set timeouts
        streams[stream_id].set_read_timeout(Some(Duration::new(60, 0))).unwrap();
        streams[stream_id].set_write_timeout(Some(Duration::new(20, 0))).unwrap();
    }

    // let the clients know which one is 'player 1'
    streams[0].write(b"1").unwrap();
    streams[1].write(b"2").unwrap();

    loop {
        for current_stream_id in 0..2 {
            match streams[current_stream_id].read(&mut received_data) {
                Ok(size) => {
                    // panic if we see a dead client
                    if received_data[0] == 0 {
                        panic!();
                    }
                    // otherwise, subtract 1 from the column int (1 was added in the client)
                    received_data[0] -= 1;

                    let mut data_to_send = [0 as u8; 3];

                    // set colour of cell
                    data_to_send[2] = current_stream_id as u8 + 1;

                    // find indexes of new cell
                    free_cells_in_column[received_data[0] as usize] -= 1;
                    data_to_send[0] = free_cells_in_column[received_data[0] as usize];
                    data_to_send[1] = received_data[0];

                    // write the instruction to the state
                    state[data_to_send[0] as usize][data_to_send[1] as usize] = data_to_send[2];

                    // if this was a winning move, indicate this in the instruction
                    if check_win(&state, &data_to_send) {
                        data_to_send[2] += 3;

                        // send the final instruction
                        streams[0].write(&data_to_send).unwrap();
                        streams[1].write(&data_to_send).unwrap();

                        // end the connection to the clients
                        break;
                    }

                    // pass the instructions to both clients
                    streams[0].write(&data_to_send).unwrap();
                    streams[1].write(&data_to_send).unwrap();

                    // clear received data buffer
                    received_data = [0 as u8; 1];
                },
                Err(_) => {
                    //println!("An error occurred or the stream timed out, terminating connection with {}", stream1.peer_addr().unwrap());
                    break;
                }
            }
        }
    }
    streams[0].shutdown(Shutdown::Both).unwrap();
    streams[1].shutdown(Shutdown::Both).unwrap();
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