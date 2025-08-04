use std::io::{BufRead, BufReader, Lines, Read};
use std::net::TcpStream;
use std::collections::HashMap;

use bytes::Buf;

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
            .map(|(_,v)| v.parse::<usize>().ok()).unwrap();

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
pub fn url_decode(s:&str) -> String {
    s.replace("%20", " ")
    .replace("%21", "!")
    .replace("%22", "\"")
    .replace("%23", "#")
    .replace("%24", "$")
    .replace("%25", "%")
    .replace("%26", "&")
    .replace("%27", "'")
    .replace("%28", "(")
    .replace("%29", ")")
    .replace("%2A", "*")
    .replace("%2B", "+")
    .replace("%2C", ",")
    .replace("%2D", "-")
    .replace("%2E", ".")
    .replace("%2F", "/")
    .replace("%3A", ":")
    .replace("%3B", ";")
    .replace("%3C", "<")
    .replace("%3D", "=")
    .replace("%3E", ">")
    .replace("%3F", "?")
    .replace("%40", "@")
    .replace("%5B", "[")
    .replace("%5C", "\\")
}


pub struct HttpResponse{
    status_code: u16,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new(status: u16, version: &str) -> Self{
        HttpResponse {
            status_code: status,
            version: version.to_string(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

}

pub fn parse_http_request(stream:&mut TcpStream) -> HttpRequest {
    let mut buf_reader: BufReader<&mut TcpStream> = BufReader::new(stream);
    let reader = buf_reader.by_ref();
    let mut lines = reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    let mut request = HttpRequest::new(request_line);
    request.parse_headers(&mut lines);
    request.parse_body(reader);
    println!("method: {},\n path: {},\n version:{},\n headers:\n {}", request.method, request.path, request.version, request.headers.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join("\n"));

    request
}


pub fn handle_connection(mut stream: TcpStream, router: &Router)-> std::io::Result<()> {
    let request = parse_http_request(&mut stream);
    router.handle(&request, &mut stream)
}


#[derive(Debug, Clone)]
pub struct RouteMatch<'a> {
    pub path_params: HashMap<String,String>,
    pub query_params: HashMap<String, String>,
    pub request: &'a HttpRequest,
}

pub type RouteHandler = fn(&RouteMatch, &mut TcpStream) -> std::io::Result<()>;

#[derive(Debug, Clone)]
struct Route {
    method: String,
    pattern: String,
    handler: RouteHandler,
}
impl Route {

    pub fn new(method: &str, pattern: &str, handler: RouteHandler) -> Self {
        Route {
            method: method.to_uppercase(),
            pattern: pattern.to_string(),
            handler,
        }
    }

    pub fn specific_score(&self) -> i32 {
        let parts : Vec<&str> = self.pattern.split("/").collect();
        let mut score = 0;
        for part in parts {
            if part.starts_with("{") && part.ends_with("}") {
                score += 1; // Path parameter increases specificity
            }else if !part.is_empty() {
                 score += 2; // Static part increases specificity
            }
        }
        score
    }
}
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    pub fn get(mut self, pattern: &str,  handler: RouteHandler) -> Self {
        self.routes.push(Route::new("GET", pattern, handler));
        self.routes.sort_by(|a, b| b.specific_score().cmp(&a.specific_score()));
        self
    }

    pub fn post(mut self, pattern:&str, handler:RouteHandler) -> Self {
        self.routes.push(Route::new("POST", pattern, handler));
        self.routes.sort_by(|a, b| b.specific_score().cmp(&a.specific_score()));
        self
    }

    pub fn route(mut self, method: &str, pattern: &str, handler: RouteHandler) -> Self {
        self.routes.push(Route::new(method, pattern, handler));
        self.routes.sort_by(|a, b| b.specific_score().cmp(&a.specific_score()));
        self
    }

    pub fn handle(&self, request: &HttpRequest, stream: &mut TcpStream) -> std::io::Result<()> {

        for route in &self.routes {
            if route.method == request.method || route.method == "*" {
                if let Some(route_match) = self.match_pattern(&route.pattern, &request){
                    return (route.handler)(&route_match, stream);
                }
            }
        }
        Ok(())
    }

    pub fn match_pattern<'a>(&self, pattern: &str, request: &'a HttpRequest) -> Option<RouteMatch<'a>> {
        let pattern_parts:Vec<&str> = pattern.split("/").collect();
        let path_parts:Vec<&str> = request.path.split("/").collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        let mut path_params = HashMap::new();

        for(pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with("{") && pattern_part.ends_with("}") {
                path_params.insert(pattern_part.to_string(),path_part.to_string());
            }else if pattern_part != path_part {
                return None;
            }
        }

        Some(RouteMatch {
            path_params,
            query_params: HashMap::new(),
            request: request,
        })
    }

}


