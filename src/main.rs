mod utils;
use crate::utils::FromStream;
use env_logger;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, ToSocketAddrs};
use std::{
    io::{Read, Write},
    net::TcpStream,
};
use utils::{HttpRequest, HttpResponse};

fn main() {
    env_logger::init();
    log::info!("Starting rust server");
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "9095".to_string());

    listen(&port);
}

fn health_handler(socket: &mut TcpStream) {
    let response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: "OK".to_string(),
    };
    utils::write_to_stream(socket, &response.to_string());
}

// function to listen incoming tcp connections on port
fn listen(port: &str) {
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to port");
    log::info!("Listening on port {}", port);
    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                log::info!("incoming request from: {:?}", addr);
                let request = HttpRequest::from_stream(&mut socket);
                if request.url.contains("/health") {
                    health_handler(&mut socket);
                    close_socket(&mut socket);
                    continue;
                }
                log::info!("request: {:?}", request.clone());
                let actual_response = request.execute().expect("failed to execute request");

                log::info!("actual response: {:?}", actual_response);
                utils::write_to_stream(&mut socket, &actual_response.to_string());
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
fn http_request(method: &str, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let url = url::Url::parse(url).expect("failed to parse url");
    let ip_address = utils::nslookup(url.host().unwrap().to_string().as_str());

    let port = url.port().unwrap_or(80);

    let socket_address = SocketAddr::new(ip_address, port);
    let mut stream = TcpStream::connect(socket_address).expect("failed to connect to server");

    let host_header = format!("Host: {}\r\n", url);
    let method = format!("{} / HTTP/1.1\r\n", method);
    utils::write_to_stream(&mut stream, &method);
    utils::write_to_stream(&mut stream, &host_header);
    utils::write_to_stream(&mut stream, "User-Agent: curl/7.64.1\r\n");
    utils::write_to_stream(&mut stream, "Accept: */*\r\n");
    utils::write_to_stream(&mut stream, "\r\n");
    // TODO: write json body
    let response = HttpResponse::from_stream(&mut stream);
    Ok(response)
}
