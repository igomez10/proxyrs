mod reqwest;

// tests
#[cfg(test)]
fn test_listen() {
    listen("5057");

    // http request to the server
    reqwest::get("http://")
}
