use dns_lookup::lookup_host;
use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
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
        let mut buffer = [0; 512];
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
        let mut body = String::new();
        let mut header_break: bool = false;
        for line in lines.clone() {
            if line.is_empty() {
                header_break = true;
            }
            if !header_break {
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
            } else {
                log::debug!("line: {}", line);
                body.push_str(line);
            }
        }

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
    pub headers: HashMap<String, String>,
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

        let mut headers = String::new();

        for header in self.headers.clone() {
            let key = header.0;
            let value = header.1;
            // TODO: fix this hack
            if key != "Content-Length" {
                headers.push_str(format!("{}: {}\r\n", key, value).as_str());
            }
        }

        log::debug!("headers string: {:?}", headers);
        format!(
            "HTTP/1.1 {} {}\r\n{}\r\n{}\r\n",
            self.status_code, reason_phrase, headers, self.body
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
        log::debug!("response: {}", response);
        let body = response.split("\r\n\r\n").last().unwrap().to_string();
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
        let mut headers_map: HashMap<String, String> = HashMap::new();
        for header in headers {
            let mut components = header.split(":");
            let key = components.next().unwrap().to_string();
            let value = components.next().unwrap().trim().to_string();
            headers_map.insert(key, value);
        }
        log::debug!("headers: {:?}", headers_map);
        Self {
            status_code,
            headers: headers_map,
            body: body,
        }
    }
}

impl HttpRequest {
    pub fn execute(&self) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let url = url::Url::parse(&self.url).expect("failed to parse url");
        let ip_address = nslookup(url.host().unwrap().to_string().as_str());

        let port = url.port().unwrap_or(80);

        let socket_address = SocketAddr::new(ip_address, port);
        let mut stream = TcpStream::connect(socket_address).expect("failed to connect to server");

        let host_header = format!("Host: {}\r\n", url);
        let method = format!("{} / HTTP/1.1\r\n", self.method);
        write_to_stream(&mut stream, &method);
        write_to_stream(&mut stream, &host_header);
        write_to_stream(&mut stream, "User-Agent: curl/7.64.1\r\n");
        write_to_stream(&mut stream, "Accept: */*\r\n");
        // // write json body
        write_to_stream(&mut stream, "\r\n");
        // write_to_stream(&mut stream, &self.body);
        // write_to_stream(&mut stream, "\r\n");

        // read from socket
        let response = HttpResponse::from_stream(&mut stream);
        Ok(response)
    }
}

// nslookup command to resolve domain name to IP address
pub fn nslookup(domain_name: &str) -> IpAddr {
    // resolve domain name to IP address
    let ips: Vec<std::net::IpAddr> =
        lookup_host(domain_name).expect(format!("nslookup failed: {}", domain_name).as_str());
    return ips[0];
}

// test nslookup with google.com
#[test]
fn test_nslookup() {
    let domain_name = "google.com";
    let ip_address = nslookup(domain_name);
    assert!(ip_address.is_ipv4());
}

pub fn write_to_stream(stream: &mut TcpStream, message: &str) {
    stream
        .write(message.as_bytes())
        .expect("failed to write to socket");
}

// function read_from_stream reads from socket and returns the result as a string
// this reads until the socket is closed
pub fn _read_from_stream(stream: &mut TcpStream) -> String {
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
