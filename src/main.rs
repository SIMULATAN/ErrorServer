use std::{fs, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}};

use crate::http_codes::get_code;

mod http_codes;
mod signals;

fn main() {
    signals::setup();
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Listening on :7878");
    let file_contents: String = fs::read_to_string("error.html").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let _ = std::panic::catch_unwind(|| handle_connection(stream, &file_contents));
    }
}

// path: /{code}.html
fn handle_connection(mut stream: TcpStream, file_contents: &String) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let empty_string = String::from("");

    let request_line = http_request.get(0).unwrap_or(&empty_string);
    let (method, path) = parse_request_line(request_line);

    let status_code = path.map(|p| p.parse::<u16>().unwrap_or(404)).unwrap_or(404);
    let status_code_message = get_code(status_code);

    let content = file_contents
        .replace("{error}", &status_code.to_string())
        .replace("{message}", status_code_message.unwrap_or("Unknown error occurred"))
        .replace("{debug}", &*(http_request.join("\r\n") + "\r\n-- Method: " + &*method.unwrap_or_else(|| "GET".to_string())));
    let length = content.len();

    let response = format!("HTTP/1.1 {status_code} {}\r\nContent-Type: text/html\r\nContent-Length: {length}\r\n\r\n{content}", status_code_message.unwrap_or("NOT FOUND"));

    stream.write_all(response.as_bytes()).unwrap();
}

fn parse_request_line(request_line: &str) -> (Option<String>, Option<String>) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().map(|x| x.to_string());
    let path = parts.next().map(|x| x.replace("/", ""));
    (method, path)
}
