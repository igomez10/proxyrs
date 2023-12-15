use env_logger;
use log::{debug, error, info, warn};
use std::net::{TcpListener, ToSocketAddrs};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() {
    env_logger::init();
    log::info!("Starting rust server");
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "9095".to_string());
    listen(&port);
}

trait FromStream {
    fn from_stream(stream: &mut TcpStream) -> Self;
}

// struct to represent HTTP Request
#[derive(Debug)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<String>,
    body: String,
}

impl FromStream for HttpRequest {
    fn from_stream(stream: &mut TcpStream) -> Self {
        let mut buffer = [0; 4096];
        stream
            .read(&mut buffer)
            .expect("failed to read from socket");
        let request = String::from_utf8_lossy(&buffer).to_string();
        let mut lines = request.lines();
        let first_line = lines.next().unwrap();
        let mut words = first_line.split_whitespace();
        let method = words.next().unwrap();
        let url = words.next().unwrap();
        let mut headers = Vec::new();
        for line in lines.clone() {
            if line.is_empty() {
                break;
            }
            headers.push(line.to_string());
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
struct HttpResponse {
    status_code: u32,
    headers: Vec<String>,
    body: String,
}

impl HttpResponse {
    fn to_string(&self) -> String {
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

// function to listen incoming tcp connections on port
fn listen(port: &str) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind to port");
    log::info!("Listening on port {}", port);
    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                log::info!("new client: {:?}", addr);
                let request = HttpRequest::from_stream(&mut socket);
                log::info!("request: {:?}", request);

                let response = HttpResponse {
                    status_code: 200,
                    headers: vec![
                        "Content-Type: application/json".to_string(),
                        "Server: rust".to_string(),
                    ],
                    body: "{\"message\": \"Not Found\"}".to_string(),
                };

                write_to_stream(&mut socket, &response.to_string());
                close_socket(&mut socket)
            }
            Err(e) => {
                log::error!("couldn't get client: {:?}", e);
            }
        }
    }
}

fn close_socket(socket: &mut TcpStream) {
    let res = socket.shutdown(std::net::Shutdown::Both);
    match res {
        Ok(_num_bytes) => {
            log::debug!("successfully closed socket");
        }
        Err(_e) => {
            log::error!("failed to close socket {:?}", _e);
        }
    }
}

fn curl(_flags: &[String]) {
    let address = _flags[0].as_str();
    let method = "GET".to_string();
    http_request(&method, address);
}

// function to execute HTTP Request
fn http_request(method: &str, url: &str) {
    let mut stream: TcpStream;
    let result = TcpStream::connect(url);
    match result {
        Ok(tcp_stream) => stream = tcp_stream,
        Err(_e) => {
            std::process::exit(1);
        }
    }

    let host_header = format!("Host: {}\r\n", url);
    let method = format!("{} / HTTP/1.1\r\n", method);
    write_to_stream(&mut stream, &method);
    write_to_stream(&mut stream, &host_header);
    write_to_stream(&mut stream, "User-Agent: curl/7.64.1\r\n");
    write_to_stream(&mut stream, "Accept: */*\r\n");
    write_to_stream(&mut stream, "\r\n");

    println!();
    // read from socket
    let response = read_from_stream(&mut stream);
    println!("{}", response);
}

fn write_to_stream(stream: &mut TcpStream, message: &str) {
    let res = stream.write(message.as_bytes());
    match res {
        Ok(num_bytes) => {
            log::debug!("successfully wrote {} bytes to socket", num_bytes)
        }
        Err(_e) => {
            log::debug!("failed to write to socket")
        }
    }
}

// function read_from_stream reads from socket and returns the result as a string
// this reads until the socket is closed
fn read_from_stream(stream: &mut TcpStream) -> String {
    // vector to store all the bytes read from socket
    let mut buffer = [0; 1024];

    log::debug!("reading from socket");
    // read from socket
    stream
        .read(&mut buffer)
        .expect("failed to read from socket");

    log::debug!("finished reading from socket");

    // convert bytes to string
    let response = String::from_utf8_lossy(&buffer).to_string();
    response
}

// nslookup command to resolve domain name to IP address
fn nslookup(_flags: &[String]) {
    let domain_name = _flags[0].as_str();
    // resolve domain name
    let result = (domain_name, 0).to_socket_addrs();
    match result {
        Ok(mut addresses) => {
            while let Some(address) = addresses.next() {
                println!("{}", address.ip());
            }
        }
        Err(_e) => {
            println!("failed to resolve domain name");
            std::process::exit(1);
        }
    }
}
