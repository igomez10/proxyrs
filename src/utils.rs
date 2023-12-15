use std::collections::HashMap;
use std::string;
use std::{io::Read, net::TcpStream};

pub trait FromStream {
    fn from_stream(stream: &mut TcpStream) -> Self;
}

// struct to represent HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    // headers i a map of string to vec string
    pub headers: HashMap<String, Vec<String>>,
    pub body: String,
}

impl FromStream for HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Self {
        let mut buffer = [0; 4096];
        stream
            .read(&mut buffer)
            .expect(" failed to read from soket");
        let request = String::from_utf8_lossy(&buffer).to_string();
        let mut lines = request.lines();
        let first_line = lines.next().unwrap();
        let mut words = first_line.split_whitespace();
        let method = words.next().unwrap();
        let url = words.next().unwrap();
        let mut headers: HashMap<String, Vec<String>> = HashMap::new();
        for line in lines.clone() {
            if line.is_empty() {
                break;
            }
            let components = line.split(":").into_iter();
            let key = components.clone().next().unwrap().to_string();
            let value = components.skip(1).next().unwrap().trim().to_string();

            match headers.get(&key) {
                Some(values) => {
                    let mut values = values.clone();
                    values.push(value);
                    headers.insert(key, values);
                }
                None => {
                    headers.insert(key, vec![value]);
                }
            }
        }
        let body = lines.collect::<Vec<&str>>().join("\n");
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers,
            body,
        }
    }
}

// struct to represent HTTP Response
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u32,
    pub headers: Vec<String>,
    pub body: String,
}

impl HttpResponse {
    pub fn to_string(&self) -> String {
        let reason_phrase = match self.status_code {
            200 => "OK",
            404 => "Not Found",
            401 => "Unauthorized",
            403 => "Forbidden",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            // Add other status codes as needed
            _ => "Unknown",
        };

        format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
            self.status_code,
            reason_phrase,
            self.headers.join("\r\n"),
            self.body
        )
    }
}

impl FromStream for HttpResponse {
    fn from_stream(stream: &mut TcpStream) -> Self {
        let mut buffer = [0; 1024];
        stream
            .read(&mut buffer)
            .expect("failed to read from socket");
        let response = String::from_utf8_lossy(&buffer).to_string();
        let mut lines = response.lines();
        let first_line = lines.next().unwrap();
        let mut words = first_line.split_whitespace();
        let _http_version = words.next().unwrap();
        let status_code = words.next().unwrap().parse::<u32>().unwrap();
        let mut headers = Vec::new();
        for line in lines.clone() {
            if line.is_empty() {
                break;
            }
            headers.push(line.to_string());
        }
        let body = lines.collect::<Vec<&str>>().join("\n");
        Self {
            status_code,
            headers,
            body,
        }
    }
}
