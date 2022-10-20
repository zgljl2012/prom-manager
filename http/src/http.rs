use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

use log::{error, info, debug};

use crate::{types::{Request, RequestMethod}, router::{Router, HttpServiceFunc}};

pub struct WebServer {
    router: Router,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    pub fn route<'a>(&mut self, name: &'a str, service: HttpServiceFunc) -> &Self {
        self.router.register(name, service);
        self
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let ip = "127.0.0.1:8080";

        let listener = TcpListener::bind(ip).expect("Unable to create listener.");
        info!("Server started on: {}{}", "http://", ip);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => match handle_connection(&self.router, stream) {
                    Ok(_) => (),
                    Err(e) => error!("Error handling connection: {}", e),
                },
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_connection(router: &Router, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    // 将流写入缓存
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap();

    match parse_request_line(&request_line) {
        Ok(request) => {
            info!("Request: {}", &request);
            let status;
            let mut body: String = "".to_string();
            match router.get(request.uri.to_str().unwrap()) {
                Some(service) => {
                    let res = service(request);
                    status = res.status;
                    body = res.body;
                },
                None => {
                    status = 404;
                }
            }

            let response = format!("{}{}",
                format!("HTTP/1.1 {:?} OK\r\n\r\n", status), 
                body);

            debug!("Response: {}", &response);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(err) => error!("Badly formatted request: {}, error: {:?}", &request_line, err),
    }

    Ok(())
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.method,
            self.uri.display(),
            self.http_version
        )
    }
}

fn parse_request_line(request: &str) -> Result<Request, Box<dyn Error>> {
    let mut parts = request.split_whitespace();

    let method = parts.next().ok_or("Method not specified")?;
    // We only accept GET requests
    if method != "GET" {
        Err(format!("Unsupported method: {method}"))?;
    }

    let uri = Path::new(parts.next().ok_or("URI not specified")?);
    let _ = uri.to_str().expect("Invalid unicode!");

    let http_version = parts.next().ok_or("HTTP version not specified")?;
    if http_version != "HTTP/1.1" && http_version != "HTTP/1.0" {
        Err(format!("Unsupported HTTP version, only support HTTP/1.0 and HTTP/1.1, but got {http_version}"))?;
    }

    Ok(Request {
        method: RequestMethod::from(method),
        uri,
        http_version,
    })
}
