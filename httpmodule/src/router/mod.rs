use std::net::TcpStream;
use std::collections::HashMap;

use crate::HttpRequest;

#[derive(Debug, Clone)]
pub struct RouteMatch<'a> {
    pub path_params: HashMap<String,String>,
    pub query_params: HashMap<String, String>,
    pub request: &'a HttpRequest,
}

pub type RouteHandler = fn(&RouteMatch, &mut TcpStream) -> std::io::Result<()>;

#[derive(Debug, Clone)]
pub struct Route {
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


