use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
};

use crate::{
    http_request::HttpRequest,
    http_response::HttpResponse,
    utils::{nslookup, write_to_stream},
};

pub struct HTTPClient {
    pub default_headers: HashMap<String, String>,
}

impl HTTPClient {
    pub fn new(default_headers: HashMap<String, String>) -> Self {
        Self { default_headers }
    }

    pub fn execute(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let ip_address = nslookup(
            request
                .url
                .host_str()
                .expect("failed to get url host: {}")
                .to_string(),
        )
        .expect("failed to resolve domain name to ip address");

        let port = request.url.port().unwrap_or(80);

        let socket_address = SocketAddr::new(ip_address, port);
        let mut stream = TcpStream::connect(socket_address).expect("failed to connect to server");

        let request_string = request.serialize();
        write_to_stream(&mut stream, &request_string).expect("failed to write to socket");
        let response = HttpResponse::from_stream(&mut stream).expect("failed to read from socket");
        Ok(response)
    }
}

// test for proxy request
#[test]
fn test_proxy_request() {
    let mut dummy_request =
        "GET http://google.com HTTP/1.1\r\nHost: http://google.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n".as_bytes();

    let request = HttpRequest::from_stream(&mut dummy_request).unwrap();
    let client = HTTPClient::new(HashMap::new());
    let response = client.execute(request).unwrap();
    assert_eq!(response.status_code.to_u32(), 400);
}
