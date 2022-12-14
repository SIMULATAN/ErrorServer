use std::{fs, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}};

use regex::Regex;

use crate::http_codes::get_code;

mod http_codes;
mod signals;

fn main() {
    signals::setup();
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Listening on :7878");
    let regex: Regex = Regex::new(r"GET /(.*) HTTP/\d.\d").unwrap();
    let file_contents: String = fs::read_to_string("error.html").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let _ = std::panic::catch_unwind(|| handle_connection(stream, &file_contents, &regex));
    }
}

// path: /{code}.html
fn handle_connection(mut stream: TcpStream, file_contents: &String, regex: &Regex) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let request_line = http_request.get(0).unwrap();
    let status_code = regex.captures_iter(&request_line).next().unwrap().get(1).unwrap().as_str().parse::<u16>().unwrap_or(1);

    let status_code_str = if status_code == 1 {"".to_string()} else {status_code.to_string()};
    let content = file_contents
        .replace("{error}", &status_code_str)
        .replace("{debug}", &*http_request.join("\r\n"))
        .replace("{message}", get_code(status_code).unwrap_or("Unknown error occurred"));
    let length = content.len();

    let http_status_code = if status_code == 1 {400} else {status_code};
    let response = format!("HTTP/1.1 {http_status_code} OK\r\nContent-Type: text/html\r\nContent-Length: {length}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}
