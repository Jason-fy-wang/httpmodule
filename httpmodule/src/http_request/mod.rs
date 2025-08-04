use std::io::{BufReader, Lines, Read};
use std::net::TcpStream;
use std::collections::HashMap;

pub mod utils;

use utils::url_decode;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub query_params: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(reqeust_line:String) -> Self {
        let mut splits = reqeust_line.split_ascii_whitespace();
        let method = splits.next().unwrap_or_else(|| {
            panic!("No method found in request line")
        }).to_string();

        let path = splits.next().unwrap_or_else(|| {
            panic!("No path found in request line")
        }).to_string();
        let version = splits.next().unwrap_or_else(|| {
            panic!("No version found in request line")
        }).to_string();

        let (path, query_params) = Self::parse_path_and_query(&path).unwrap();

        HttpRequest {
            method,
            path,
            version,
            headers: Vec::new(),
            body: String::new(),
            query_params,
        }
    }

    pub fn add_header(&mut self, key:String, value:String){
        self.headers.push((key, value));
    }

    pub fn parse_headers(&mut self, lines: &mut Lines<&mut BufReader<&mut TcpStream>>) {
        while let Some(line) = lines.next() {
            let line = line.unwrap();
            if line.is_empty() {
                break; // End of headers
            }
            let mut parts = line.splitn(2, ':');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                self.add_header(key.trim().to_string(), value.trim().to_string());
            }
        }
    }

    pub fn parse_body(&mut self, buf: &mut BufReader<&mut TcpStream>) {
        let length = self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case("Content-Length"))
        .and_then(|(_,v)| v.parse::<usize>().ok());

        if let Some(len)  = length {
            if len > 0 {
                let mut buffer = vec![0u8; len];
                buf.read(&mut buffer).unwrap();
                self.body = String::from_utf8_lossy(&buffer).to_string();
            }
        }
    }

    pub fn parse_query_string(query_string:&str) -> HashMap<String, String> {
        let mut query_params = HashMap::new();
        for pair in query_string.split("&") {
            if let Some((key, value)) = pair.split_once("=") {
                let key = url_decode(key);
                let value = url_decode(value);
                query_params.insert(key, value);
            }
        }
        query_params
    }

    pub fn parse_path_and_query(full_path: &str) -> Option<(String, HashMap<String, String>)> {
        if let Some((path, query_string)) = full_path.split_once("?"){
            let query_params = Self::parse_query_string(query_string);
            Some((path.to_string(), query_params))
        }else {
            return Some((full_path.to_string(),HashMap::new()));
        }
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_request() {
        let request_str = "GET /api/v1/test?code=123 HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n";

        let mut lines = request_str.lines();
        let request_line = lines.next().unwrap();
        let reqeust = HttpRequest::new(request_line.to_string());

        assert_eq!(reqeust.method, "GET");
        assert_eq!(reqeust.path, "/api/v1/test");
        assert_eq!(reqeust.version, "HTTP/1.1");
        assert_eq!(reqeust.query_params.get("code"), Some(&"123".to_string()));
    }
}