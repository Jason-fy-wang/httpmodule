use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
pub mod http_request;
pub mod router;
pub mod http_response;

use router::{Router};

use http_request::HttpRequest;

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

