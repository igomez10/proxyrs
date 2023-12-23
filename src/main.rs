mod http_request;
mod http_response;
mod utils;
extern crate dotenv;
use env_logger;
use http_request::HttpRequest;
use http_response::{HttpResponse, StatusCode};
use std::collections::HashMap;
use std::net::TcpStream;
use std::net::{SocketAddr, TcpListener};

use dotenv::dotenv;

use crate::http_request::HTTPClient;

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
    let response = http_response::HttpResponse {
        status_code: StatusCode::OK,
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
    let client = HTTPClient::new(HashMap::new());

    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                log::info!("incoming request from: {:?}", addr);
                let request_string = utils::read_from_stream(&mut socket);
                let request = match HttpRequest::from_string(&request_string) {
                    Ok(request) => request,
                    Err(e) => {
                        log::error!("failed to parse request: {:?}", e);
                        let response = HttpResponse {
                            status_code: StatusCode::InvalidRequest,
                            headers: HashMap::new(),
                            body: "Bad Request".to_string(),
                        };
                        utils::write_to_stream(&mut socket, &response.to_string());
                        close_socket(socket);
                        continue;
                    }
                };

                if request.url.as_str().contains("/health")
                    || request.url.as_str().contains("/favicon.ico")
                // || !request.headers.contains_key("Proxy-Connection")
                {
                    log::debug!("ignore request: {:?}", request.clone());
                    health_handler(&mut socket);
                    close_socket(socket);
                    continue;
                }

                let actual_response = client.execute(request).expect("failed to execute request");
                utils::write_to_stream(&mut socket, &actual_response.to_string());
                close_socket(socket)
            }
            Err(e) => {
                log::error!("couldn't get client: {:?}", e);
            }
        }
    }
}

fn close_socket(socket: TcpStream) {
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
