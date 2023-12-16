mod utils;
use crate::utils::FromStream;
use env_logger;
use std::collections::HashMap;
use std::net::TcpStream;
use std::net::{SocketAddr, TcpListener};
use utils::{HttpRequest, HttpResponse};
extern crate dotenv;
use dotenv::dotenv;
use std::env;

fn main() {
    match dotenv().ok() {
        Some(_) => println!("dotenv loaded"),
        None => println!("dotenv not loaded"),
    }
    env_logger::init();
    println!("Starting rust server");

    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "9095".to_string());
    listen(&address, &port);
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
fn listen(address: &str, port: &str) {
    let listener =
        TcpListener::bind(format!("{}:{}", address, port)).expect("Failed to bind to port");
    log::info!("Listening on port {}", port);
    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                log::info!("incoming request from: {:?}", addr);
                let request = HttpRequest::from_stream(&mut socket);
                if request.url.contains("/health")
                    || request.url.contains("/favicon.ico")
                    || !request.headers.contains_key("Proxy-Connection")
                {
                    log::debug!("ignore request: {:?}", request.clone());
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

// function to execute HTTP Request
fn _http_request(method: &str, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
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
