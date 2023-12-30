use crate::http_method::Method;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::status_code::StatusCode;
use dns_lookup::lookup_host;

use std::collections::HashMap;
use std::error;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;
use std::str::FromStr;

// nslookup command to resolve domain name to IP address
pub fn nslookup(domain_name: String) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // resolve domain name to IP address
    match lookup_host(domain_name.as_str()) {
        Ok(ips) => Ok(ips[0]),
        Err(e) => Err(e.into()),
    }
}

// test nslookup with localhost
#[test]
fn test_nslookup() {
    let domain_name = String::from("localhost");
    let ip_address = nslookup(domain_name);
    assert!(ip_address.is_ok());
}

pub fn write_to_stream(stream: &mut TcpStream, message: &str) -> Result<(), Box<dyn error::Error>> {
    match stream.write(message.as_bytes()) {
        Ok(num_bytes) => {
            if num_bytes != message.len() {
                log::error!("failed to write all bytes to socket");
                return Err("failed to write all bytes to socket".into());
            }
            Ok(())
        }
        Err(e) => {
            log::error!("failed to write to socket {:?}", e);
            Err(e.into())
        }
    }
}

pub fn read_request(stream: &mut dyn Read) -> Result<HttpRequest, Box<dyn std::error::Error>> {
    let mut buf_reader = BufReader::new(stream);
    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line)?;

    let words_first_line: Vec<&str> = first_line.split_whitespace().collect();
    let method = Method::from_str(words_first_line[0])?;
    let resource = words_first_line[1];
    let _protocol = words_first_line[2];

    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line = String::new();
        let num_bytes = buf_reader.read_line(&mut line)?;
        if num_bytes == 0 {
            break;
        }
        if line == "\r\n" {
            break;
        }

        let components: Vec<&str> = line.splitn(2, ':').collect();
        let key: String = components[0].to_string();
        let value: String = components[1].trim().to_string();
        headers.insert(key, value);
    }

    let url = match url::Url::parse(resource) {
        Ok(url) => url,
        Err(_e) => {
            // if url is not valid, then it is a path
            let host_header = headers.get("Host").expect("missing host header");
            let host_url = url::Url::parse(format!("http://{}", host_header).as_str()).unwrap();
            host_url.join(resource).unwrap()
        }
    };

    // ready body. read content length bytes
    let mut body = String::new();
    if let Some(content_length) = headers.get("Content-Length") {
        let content_length = content_length.parse::<usize>().unwrap();
        let mut buffer = vec![0; content_length];
        buf_reader.read_exact(&mut buffer)?;
        body = String::from_utf8_lossy(&buffer).to_string();
    }

    let request = HttpRequest {
        body,
        headers,
        method,
        url,
    };

    Ok(request)
}

pub fn read_response(stream: &mut dyn Read) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    // input: "HTTP/1.1 200 OK\r\nDate: Mon, 23 May 2023 22:38:34 GMT\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: 132\r\n\r\n<html>\r\n<head>\r\n<title>An Example Page</title>\r\n</head>\r\n<body>\r\nHello World, this is a very simple HTML document.\r\n</body>\r\n</html>".to_string(),
    let mut buf_reader = BufReader::new(stream);
    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line)?;

    let words_first_line: Vec<&str> = first_line.split_whitespace().collect();
    let _protocol = words_first_line[0];
    let status_code = StatusCode::from_u32(words_first_line[1].parse()?)?;

    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line = String::new();
        let num_bytes = buf_reader.read_line(&mut line)?;
        if num_bytes == 0 {
            break;
        }
        if line == "\r\n" {
            break;
        }

        let components: Vec<&str> = line.splitn(2, ':').collect();
        let key: String = components[0].to_string();
        let value: String = components[1].trim().to_string();
        headers.insert(key, value);
    }

    // ready body. read content length bytes
    let mut body = String::new();
    if let Some(content_length) = headers.get("Content-Length") {
        let content_length = content_length.parse::<usize>().unwrap();
        let mut buffer = vec![0; content_length];
        buf_reader.read_exact(&mut buffer)?;
        body = String::from_utf8_lossy(&buffer).to_string();
    }

    let response = HttpResponse {
        body,
        headers,
        status_code,
    };

    Ok(response)
}

#[test]
fn test_read_request() {
    let tests = vec![
        (
            "POST /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
            ("POST", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
        ),
        (
            "GET /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n",
            ("GET", "/users/1", "google.com", "curl/7.64.1", "*/*", "unused", "unused")
        ),
        (
            "PUT /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
            ("PUT", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
        ),
        (
            "DELETE /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
            ("DELETE", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
        ),
        (
            "OPTIONS /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
            ("OPTIONS", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
        ),
        (
            "HEAD /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
            ("HEAD", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
        )
    ];

    for (input, (method, path, host, user_agent, accept, content_length, body)) in tests {
        let mut dummy_request = input.as_bytes();
        let request = read_request(&mut dummy_request).unwrap();
        assert_eq!(method, request.method.to_string());
        assert_eq!(path, request.url.path());
        assert_eq!(host, request.headers["Host"]);
        assert_eq!(user_agent, request.headers["User-Agent"]);
        assert_eq!(accept, request.headers["Accept"]);
        if request.method != Method::Get {
            assert_eq!(content_length, request.headers["Content-Length"]);
            assert_eq!(body, request.body);
        }
    }
}

#[test]
fn test_read_request_lifecycle() {
    let tests_requests: Vec<HttpRequest> = vec![
        HttpRequest {
            method: http_method::Method::Get,
            url: url::Url::parse("http://example.com").unwrap(),
            headers: HashMap::from([
                ("Content-Length".to_string(), "5".to_string()),
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "curl".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
            ]),
            body: "hello".to_string(),
        },
        HttpRequest {
            method: http_method::Method::Post,
            url: url::Url::parse("http://example.com").unwrap(),
            headers: HashMap::from([
                ("Content-Length".to_string(), "14".to_string()),
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "curl".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
            ]),
            body: "<h1>hello</h1>".to_string(),
        },
    ];

    // let tests = vec![
    //     (
    //         "POST /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
    //         ("POST", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
    //     ),
    //     (
    //         "GET /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n",
    //         ("GET", "/users/1", "google.com", "curl/7.64.1", "*/*", "unused", "unused")
    //     ),
    //     (
    //         "PUT /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
    //         ("PUT", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
    //     ),
    //     (
    //         "DELETE /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
    //         ("DELETE", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
    //     ),
    //     (
    //         "OPTIONS /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
    //         ("OPTIONS", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
    //     ),
    //     (
    //         "HEAD /users/1 HTTP/1.1\r\nHost: google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n<h1>Hello World</h1>",
    //         ("HEAD", "/users/1", "google.com", "curl/7.64.1", "*/*", "20", "<h1>Hello World</h1>")
    //     )
    // ];

    for input in tests_requests {
        let serialized = input.serialize();
        let mut dummy_request = serialized.as_bytes();
        let request = read_request(&mut dummy_request).unwrap();
        assert_eq!(input.method, request.method);
        assert_eq!(input.url, request.url);
        assert_eq!(input.headers["Host"], request.headers["Host"]);
        assert_eq!(input.headers["User-Agent"], request.headers["User-Agent"]);
        assert_eq!(input.headers["Accept"], request.headers["Accept"]);
        if request.method != Method::Get {
            assert_eq!(
                input.headers["Content-Length"],
                request.headers["Content-Length"]
            );
            assert_eq!(input.body, request.body);
        }
    }
}

#[test]
fn test_read_response_lifecycle() {
    let tests_responses: Vec<HttpResponse> = vec![
        HttpResponse {
            status_code: StatusCode::OK,
            headers: HashMap::from([
                ("Content-Length".to_string(), "5".to_string()),
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "curl".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
            ]),
            body: "hello".to_string(),
        },
        HttpResponse {
            status_code: StatusCode::OK,
            headers: HashMap::from([
                ("Content-Length".to_string(), "14".to_string()),
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "curl".to_string()),
                ("Accept".to_string(), "*/*".to_string()),
            ]),
            body: "<h1>hello</h1>".to_string(),
        },
    ];

    for input in tests_responses {
        let serialized = input.serialize();
        let mut dummy_response = serialized.as_bytes();
        let response = read_response(&mut dummy_response).unwrap();
        assert_eq!(input.status_code, response.status_code);
        assert_eq!(input.headers["Host"], response.headers["Host"]);
        assert_eq!(input.headers["User-Agent"], response.headers["User-Agent"]);
        assert_eq!(input.headers["Accept"], response.headers["Accept"]);
    }
}
