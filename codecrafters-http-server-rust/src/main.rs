use std::net::{TcpListener, TcpStream};
use std::io::{Write};

use httpmodule;

fn home_handler(_route: &httpmodule::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn api_message_handler(route: &httpmodule::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let message = route.path_params.get("{message}")
        .map(|s| s.as_str())
        .unwrap_or("default");

    let response  = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, {}!", message);
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn api_code_handler(route: &httpmodule::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    print!("route: {:?}", route);
    let code = route.query_params.get("code")
        .map(|s| s.as_str())
        .unwrap_or("default");

    let version = route.query_params.get("version")
        .map(|s| s.as_str())
        .unwrap_or("1.0");    

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nCode is: {}, version: {}!", code, version);
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn build_router() -> httpmodule::Router {
    httpmodule::Router::new()
    .get("/", home_handler)
    .get("/api/v1/{message}", api_message_handler)
    .get("/api/v1/code", api_code_handler)
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let router = build_router();
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                if let Err(e) = httpmodule::handle_connection(stream, &router) {
                    println!("error handling connection: {}", e);
                }
            }
            Err(e) => {
                println!("error accepting connection: {}", e);
            }
        }
    }
}


