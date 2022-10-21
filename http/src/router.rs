use std::collections::HashMap;
use log::error;

use crate::{Request, Response};
pub type HttpServiceFunc = fn(Request) ->  Response;

pub struct Router {
    routes: HashMap<String, HttpServiceFunc>
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new()
        }
    }
    pub fn register<'a>(&mut self, route_name: &'a str, route_func: HttpServiceFunc) -> &Self {
        if self.routes.contains_key(route_name) {
            error!("Route {route_name} already registered");
        }
        self.routes.insert(route_name.to_string(), route_func);
        self
    }
    pub fn get(&self, route_name: &str) -> Option<&HttpServiceFunc> {
        self.routes.get(route_name)
    }
}
