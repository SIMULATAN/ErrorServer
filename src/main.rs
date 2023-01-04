use std::{fs, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}};
use std::sync::Arc;

use threads::ThreadPool;

use crate::http_codes::get_code;
use crate::threads::Timing;

mod http_codes;
mod signals;
mod threads;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Listening on :7878");
    let file_contents: String = fs::read_to_string("error.html").unwrap();
    let file_contents_working: &'static str = Box::leak(file_contents.into_boxed_str());

    let pool = ThreadPool::build(10).unwrap();
    let pool_arc = Arc::new(pool);
    signals::setup(Arc::clone(&pool_arc));

    let pool_arc = Arc::clone(&pool_arc);
    for stream in listener.incoming() {
        pool_arc.execute(|timing| {
            // ignore unwrapping errors
            let result = handle_connection(stream.unwrap(), file_contents_working, timing);
            if result.is_err() {
                println!("Error: {:?}", result);
            }
        });
    }
}

// path: /{code}.html
fn handle_connection(mut stream: TcpStream, file_contents: &str, timing: &mut Timing) -> Result<(), std::io::Error> {
    timing.start();
    let mut received: Vec<u8> = vec![];
    loop {
        let buf = &mut [0; 4096];
        match stream.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                received.extend_from_slice(&buf[..n]);
                if n < buf.len() && buf[n] == 0 {
                    break;
                }
            },
            Err(e) => eprintln!("{e}"),
        };
    }
    timing.end(0);

    timing.start();
    // read until the first newline into a string
    let mut line = String::new();
    received.windows(2).position(|window| window == vec!(0x0, 0xA, 0x0, 0xD, 0x0, 0xA, 0x0, 0xD)).map(|pos| {
        line = String::from_utf8_lossy(&received[..pos]).to_string();
    });
    let request_line = line.trim();

    let (_method, path) = parse_request_line(request_line);
    timing.end(1);

    timing.start();
    let status_code = path.map(|p| p.parse::<u16>().unwrap_or(404)).unwrap_or(404);
    let status_code_message = get_code(status_code);
    timing.end(2);

    timing.start();
    let mut content = file_contents
        .replace("{error}", &status_code.to_string())
        .replace("{message}", status_code_message.unwrap_or("Unknown error occurred"));
    timing.end(3);

    timing.start();
    let mut result;
    unsafe {
        result = content.as_mut_vec();
        let bytes_of_placeholder = b"{debug}";
        // substitute in the debug placeholder with "received"
        let placeholder_index = result.windows(bytes_of_placeholder.len()).position(|window| window == bytes_of_placeholder).unwrap();
        result.splice(placeholder_index..placeholder_index + bytes_of_placeholder.len(), received.into_iter());
    }
    let length = result.len();
    let response = format!("HTTP/1.1 {status_code} {}\r\nContent-Type: text/html\r\nContent-Length: {length}\r\n\r\n", status_code_message.unwrap_or("NOT FOUND"));
    timing.end(4);

    timing.start();
    let result = stream.write_all(&*[response.as_bytes(), result.as_slice()].concat());
    timing.end(5);
    result
}

fn parse_request_line(request_line: &str) -> (Option<String>, Option<String>) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().map(|x| x.to_string());
    let path = parts.next().map(|x| x.replace("/", ""));
    (method, path)
}
