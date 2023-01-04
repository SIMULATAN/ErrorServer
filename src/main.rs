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
            let _ = handle_connection(stream.unwrap(), file_contents_working, timing);
        });
    }
}

// path: /{code}.html
fn handle_connection(mut stream: TcpStream, file_contents: &str, timing: &mut Timing) -> Result<(), std::io::Error> {
    timing.start();
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    timing.end(0);

    let empty_string = String::from("");

    timing.start();
    let request_line = http_request.get(0).unwrap_or(&empty_string);
    let (method, path) = parse_request_line(request_line);
    timing.end(1);

    timing.start();
    let status_code = path.map(|p| p.parse::<u16>().unwrap_or(404)).unwrap_or(404);
    let status_code_message = get_code(status_code);
    timing.end(2);

    timing.start();
    let content = file_contents
        .replace("{error}", &status_code.to_string())
        .replace("{message}", status_code_message.unwrap_or("Unknown error occurred"))
        .replace("{debug}", &*(http_request.join("\r\n") + "\r\n-- Method: " + &*method.unwrap_or_else(|| "GET".to_string())));
    let length = content.len();
    timing.end(3);

    timing.start();
    let response = format!("HTTP/1.1 {status_code} {}\r\nContent-Type: text/html\r\nContent-Length: {length}\r\n\r\n{content}", status_code_message.unwrap_or("NOT FOUND"));
    timing.end(4);

    timing.start();
    let result = stream.write_all(response.as_bytes());
    timing.end(5);
    result
}

fn parse_request_line(request_line: &str) -> (Option<String>, Option<String>) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().map(|x| x.to_string());
    let path = parts.next().map(|x| x.replace("/", ""));
    (method, path)
}
