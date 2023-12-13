use std::error::Error;
use std::net::ToSocketAddrs;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() {
    println!("Hello, world!");
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
    match response {
        Ok(response) => {
            println!("{}", response);
        }
        Err(_e) => {
            println!("failed to read from socket");
        }
    }
}

fn write_to_stream(stream: &mut TcpStream, message: &str) {
    let res = stream.write(message.as_bytes());
    match res {
        Ok(num_bytes) => {
            println!("successfully wrote {} bytes to socket", num_bytes)
        }
        Err(_e) => {
            println!("failed to write to socket")
        }
    }
}

// function read_from_stream reads from socket and returns the result as a string
// this reads until the socket is closed
fn read_from_stream(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    // vector to store all the bytes read from socket
    let mut buffer = Vec::new();

    // read from socket
    let result = stream.read_to_end(&mut buffer);

    // check if read was successful
    match result {
        Ok(_num_bytes) => {
            // convert bytes to string
            let response = String::from_utf8(buffer).unwrap();
            Ok(response)
        }
        Err(e) => Err(e),
    }
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
