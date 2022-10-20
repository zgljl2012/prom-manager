use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

use log::{error, info, debug};
use reqwest::Client;
use tokio::runtime::Runtime;

use crate::{types::{Request, RequestMethod}, router::{Router, HttpServiceFunc}, Response};

pub struct WebServer {
    rt: Runtime,
    client: Client,
    router: Router,
    host: String,
    port: u16,
}

impl WebServer {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            rt: tokio::runtime::Runtime::new().unwrap(),
            client: reqwest::Client::new(),
            router: Router::new(),
        }
    }

    pub fn bind(mut self, host: String, port: u16) -> Self {
        self.host = host;
        self.port = port;
        self
    }

    pub fn route<'a>(mut self, name: &'a str, service: HttpServiceFunc) -> Self {
        self.router.register(name, service);
        self
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let ip = "127.0.0.1:8080";

        let listener = TcpListener::bind(ip).expect("Unable to create listener.");
        info!("Server started on: {}{}", "http://", ip);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => match handle_connection(&self.rt, &self.client, &self.router, stream) {
                    Ok(_) => (),
                    Err(e) => error!("Error handling connection: {}", e),
                },
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_connection(rt: &Runtime, client: &Client, router: &Router, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    const BATCH_SIZE: usize = 1024;
    let mut buffer = [0; BATCH_SIZE];
    // 将流写入缓存
    let mut text = String::new();
    loop {
        let nsize = stream.read(&mut buffer).unwrap();
        let s = String::from_utf8(buffer[..nsize].to_vec()).unwrap();
        text = format!("{}{}", text, s);
        if nsize < BATCH_SIZE {
            break;
        }
    }
    let request_line = text;
    let response = match parse_request_line(&rt, &client, &request_line) {
        Ok(request) => {
            info!("Request: {}", &request);
            match router.get(request.uri.to_str().unwrap()) {
                Some(service) => service(request),
                None => Response {status: 404, body: "".to_string() }
            }
        }
        Err(err) => {
            Response {
                status: 400,
                body: format!("Badly formatted request: {}, error: {:?}", &request_line, err)
            }
        },
    };
    let response_ = format!("{}{}",
        format!("HTTP/1.1 {:?} OK\r\n\r\n", response.status), 
        response.body);

    debug!("Response: {}", &response_);
    stream.write(response_.as_bytes()).unwrap();
    stream.flush().unwrap();

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

fn parse_request_line<'a>(rt: &'a Runtime, client: &'a Client, request: &'a str) -> Result<Request<'a>, Box<dyn Error>> {
    let mut parts = request.split_whitespace();
    // Parse the http method
    let method_s = parts.next().ok_or("Method not specified")?;
    // We only accept GET requests
    if method_s != "GET" && method_s != "POST" {
        Err(format!("Unsupported method: {method_s}"))?;
    }
    // Parse URL
    let uri = Path::new(parts.next().ok_or("URI not specified")?);
    let _ = uri.to_str().expect("Invalid unicode!");
    let method = RequestMethod::from(method_s);
    // Parse http version
    let http_version = parts.next().ok_or("HTTP version not specified")?;
    if http_version != "HTTP/1.1" && http_version != "HTTP/1.0" {
        Err(format!("Unsupported HTTP version, only support HTTP/1.0 and HTTP/1.1, but got {http_version}"))?;
    }
    // TODO: Parse headers
    // Parse request
    match method {
        RequestMethod::GET => {
            Ok(Request {
                rt: &rt,
                client: &client,
                method,
                uri,
                http_version,
                body: None,
            })
        },
        RequestMethod::POST => {
            // Parse request body
            let mut lines = request.split_terminator("\r\n");
            let mut body = String::new();
            let mut start = false;
            while let Some(line) = lines.next() {
                if line == "" {
                    start = true;
                }
                if start {
                    body = format!("{}{}", body, line);
                }
            }
            Ok(Request {
                rt: &rt,
                client: &client,
                method,
                uri,
                http_version,
                body: Some(body),
            })
        }
    }
}
