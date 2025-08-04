use std::net::{TcpListener, TcpStream};
use std::io::{Write};

use httpmodule::{router,http_response::HttpResponse};

fn home_handler(_route: &router::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let response = HttpResponse::new(200)
        .add_header("Content-Type".to_string(), "text/plain".to_string())
        .set_body("Hello, World!".to_string())
        .to_string();

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn api_message_handler(route: &router::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let message = route.path_params.get("{message}")
        .map(|s| s.as_str())
        .unwrap_or("default");
    
    let response = HttpResponse::new(200)
        .add_header("Content-Type".to_string(), "text/plain".to_string())
        .set_body(format!("Hello, {}!", message))
        .to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn api_code_handler(route: &router::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let code = route.query_params.get("code")
        .map(|s| s.as_str())
        .unwrap_or("default");

    let version = route.query_params.get("version")
        .map(|s| s.as_str())
        .unwrap_or("1.0");    

    let response = HttpResponse::new(200)
        .add_header("Content-Type".to_string(), "text/plain".to_string())
        .set_body(format!("Code is: {}, version: {}!", code, version))
        .to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn api_code_post_handler(route: &router::RouteMatch, stream: &mut TcpStream) -> std::io::Result<()> {
    let code = route.query_params.get("code")
        .map(|s| s.as_str())
        .unwrap_or("default");

    let version = route.query_params.get("version")
        .map(|s| s.as_str())
        .unwrap_or("1.0");    
    let body = &route.request.body;
    let response = HttpResponse::new(200)
        .add_header("Content-Type".to_string(), "text/plain".to_string())
        .set_body(format!("Code is: {}, version: {}, body: {}!", code, version, body))
        .to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}


fn build_router() -> router::Router {
    router::Router::new()
    .get("/", home_handler)
    .get("/api/v1/{message}", api_message_handler)
    .get("/api/v1/code", api_code_handler)
    .post("/api/v1/code", api_code_post_handler)
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


