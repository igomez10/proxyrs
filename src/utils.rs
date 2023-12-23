use dns_lookup::lookup_host;
use std::io::Write;
use std::net::IpAddr;
use std::time::Duration;
use std::{io::Read, net::TcpStream};

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
pub fn read_from_stream(stream: &mut TcpStream) -> String {
    // vector to store all the bytes read from socket
    let mut buffer = [0; 2048];

    log::debug!("reading from socket");
    // read from socket
    stream
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("failed to set read timeout");

    stream
        .read(&mut buffer)
        .expect("failed to read from socket");

    log::debug!("finished reading from socket");

    // convert bytes to string
    let response = String::from_utf8_lossy(&buffer).to_string();
    response
}
