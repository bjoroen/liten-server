use std::{
    arch::asm,
    io::{self, Read, Write},
    net::TcpListener,
    os::fd::AsRawFd,
};

use epoll::{ControlOptions::*, Event, Events};

// use liten_http::Request;

enum ConnectionState {
    Read {
        request: [u8; 1024],
        read: usize,
    },
    Write {
        response: &'static [u8],
        written: usize,
    },
    Flush,
}

fn main() {
    let epoll = epoll::create(false).unwrap();
    let listener =
        TcpListener::bind("127.0.0.1:3000").expect("Could not start server on port 3000");
    listener.set_nonblocking(true).unwrap();

    // Add the listener to epoll
    let event = Event::new(Events::EPOLLIN, listener.as_raw_fd() as _);
    epoll::ctl(epoll, EPOLL_CTL_ADD, listener.as_raw_fd(), event).unwrap();

    loop {
        let mut events = [Event::new(Events::empty(), 0); 1024];
        let timeout = -1;
        let num_events = epoll::wait(epoll, timeout, &mut events).unwrap();

        for event in &events[..num_events] {}
    }

    let mut connections = Vec::new();

    loop {
        match listener.accept() {
            Ok((connection, _)) => {
                connection.set_nonblocking(true).unwrap();
                let state = ConnectionState::Read {
                    request: [0u8; 1024],
                    read: 0,
                };
                connections.push((connection, state))
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                eprintln!("Error: {e}")
            }
        };

        let mut completed = Vec::new();

        'next: for (i, (connection, state)) in connections.iter_mut().enumerate() {
            if let ConnectionState::Read { request, read } = state {
                loop {
                    match connection.read(&mut request[*read..]) {
                        Ok(0) => {
                            eprintln!("client disconnected...");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(n) => *read += n,

                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue 'next,
                        Err(e) => eprintln!("Error: {e}"),
                    }

                    // Dose not work if post requests have a body
                    if request.get(*read - 4..*read) == Some(b"\r\n\r\n") {
                        break;
                    }
                }

                let request_str = String::from_utf8_lossy(&request[..*read]);
                println!("{request_str}");

                let response = concat!(
                    "HTTP/1.1 200 OK\r\n",
                    "Content-Length: 12\n",
                    "Connection: close\r\n\r\n",
                    "Hello world!"
                );

                *state = ConnectionState::Write {
                    response: response.as_bytes(),
                    written: 0,
                }
            }
            if let ConnectionState::Write { response, written } = state {
                loop {
                    match connection.write(&response[*written..]) {
                        Ok(0) => {
                            eprintln!("client disconnected...");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(n) => *written += n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue 'next,
                        Err(e) => eprintln!("Error: {e}"),
                    }

                    if *written == response.len() {
                        break;
                    }
                }

                *state = ConnectionState::Flush;
            }
            if let ConnectionState::Flush = state {
                match connection.flush() {
                    Ok(_) => completed.push(i),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue 'next,
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
        }

        for i in completed.into_iter().rev() {
            connections.remove(i);
        }
    }
}
