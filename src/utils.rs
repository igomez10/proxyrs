use dns_lookup::lookup_host;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;
use std::time::Duration;

// nslookup command to resolve domain name to IP address
pub fn nslookup(domain_name: String) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // resolve domain name to IP address
    let ips = lookup_host(domain_name.as_str())
        .expect(format!("nslookup failed: {}", domain_name).as_str());

    Ok(ips[0])
}

// test nslookup with localhost
#[test]
fn test_nslookup() {
    let domain_name = String::from("localhost");
    let ip_address = nslookup(domain_name);
    assert!(ip_address.is_ok());
}

pub fn write_to_stream(stream: &mut TcpStream, message: &str) {
    stream
        .write(message.as_bytes())
        .expect("failed to write to socket");
}

// function read_from_stream reads from socket and returns the result as a string
// this reads until the socket is closed
pub fn read_from_stream(stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
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
    Ok(response)
}
