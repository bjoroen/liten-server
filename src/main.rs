use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

enum HttpVerb {
    GET,
    POST,
}

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:3000").expect("Could not start server on port 3000");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let buf_read = BufReader::new(&mut stream);
            let http_request: Vec<_> = buf_read
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            println!("{:#?}", http_request);

            let verb = get_verb(&http_request[0]);

            match verb {
                HttpVerb::GET => handle_get(stream),
                HttpVerb::POST => todo!(),
            }
        } else {
            // Log issue
            todo!()
        }

        println!("connection...")
    }
}

fn get_verb(req_string: &str) -> HttpVerb {
    let verb = req_string.split_once(" ").unwrap();
    match verb.0 {
        "GET" => HttpVerb::GET,
        "POST" => HttpVerb::POST,
        _ => todo!(),
    }
}

fn handle_get(mut stream: std::net::TcpStream) {
    let status_line = "HTTP/1.1 200 OK";

    let html = match fs::read_to_string("./src/index.html") {
        Ok(v) => v,
        Err(e) => panic!("could not read file: {e}"),
    };

    let length = html.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{html}");

    stream
        .write_all(response.as_bytes())
        .unwrap_or_else(|e| eprintln!("Could not write to stream: {e}"));
}
