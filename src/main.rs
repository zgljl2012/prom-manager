use actix_web::{get, web::{self, Data}, App, HttpServer, Responder};
use clap::{arg, Command, ArgMatches};
use env_logger::{Builder, Target};
use log::{debug, error, info};

use crate::state::AppState;
mod prometheus;
mod state;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

fn cli () -> Command {
    let port_arg = arg!(-p - -port <PORT> "Specify a port to listen").value_parser(clap::value_parser!(u16).range(3000..)).required(false);
    let host_arg = arg!(-H - -host <HOST> "Specify a host to listen").required(false);
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
               .about("Start a node")
               .arg(&port_arg)
               .arg(&host_arg)
        )
}

struct ServerOpts {
    pub host: String,
    pub port: u16,
}

fn get_server_opts(sub_matches: &ArgMatches) -> ServerOpts {
    let port = sub_matches.get_one::<u16>("port");
    let p: u16 = match port {
        Some(port) => port.clone(),
        None => 8080,
    };
    let host = sub_matches.get_one::<String>("host");
    let h = match host {
        Some(host) => host.clone(),
        None => "127.0.0.1".to_string(),
    };
    ServerOpts {
        host: h,
        port: p,
    }
}
    

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();
    debug!("Starting environment logger");
    let matches = cli().get_matches();
    let state = Data::new(AppState{
        wechat_robot: Some("https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=d0b8ea9c-bebc-4e71-a85e-d1cd3b1f101e".to_string()),
    });
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let server_opts = get_server_opts(sub_matches);
            info!("Start listen on http://{}:{}", server_opts.host, server_opts.port);
            HttpServer::new(move || {
                App::new()
                    .app_data(state.clone())
                    .route("/hello", web::get().to(|| async { "Hello World!" }))
                    .service(prometheus::prometheus_hook)
                    .service(greet)
            })
            .bind((server_opts.host, server_opts.port))?
            .run()
            .await.unwrap();
        },
        _ => error!("not implemented"),
    };
    Ok(())
}
