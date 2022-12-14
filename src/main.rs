use std::{sync::{Arc, Mutex}, path::Path, fs};

use actix_web::{get, web::{self, Data}, App, HttpServer, Responder};
use clap::{arg, Command, ArgMatches};
use env_logger::{Builder, Target};
use log::{debug, error, info, warn};

use crate::state::{AppState, MachineManager};
mod prometheus;
pub mod state;
mod machine;
mod service;

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
               .arg(arg!(--"wechat-robot" <WECHAT_ROBOT> "Hook URL of the wechat robot"))
               .arg(arg!(--"prom-dir" <PROM_DIR> "Directory containing config_machines.json and config_services.json"))
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
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let server_opts = get_server_opts(sub_matches);
            info!("Start listen on http://{}:{}", server_opts.host, server_opts.port);
            let prom_dir = match sub_matches.get_one::<String>("prom-dir") {
                Some(s) => {
                    let p = Path::new(s);
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    } else {
                        if !p.is_dir() {
                            panic!("{} is not a directory", p.to_str().unwrap());
                        }
                    }
                    s.clone()
                },
                None => "./".to_string(),
            };
            let config_machines = Path::new(&prom_dir).join("config_machines.json").to_str().unwrap().to_string();
            let config_services = Path::new(&prom_dir).join("config_services.json").to_str().unwrap().to_string();
            let state = Data::new(AppState{
                machine_manager: Arc::new(Mutex::new(MachineManager::new(config_machines))),
                service_manager: Arc::new(Mutex::new(MachineManager::new(config_services))),
                wechat_robot: match sub_matches.get_one::<String>("wechat-robot") {
                    Some(bot) => Some(bot.clone()),
                    None => {
                        warn!("You not specified a wechat robot");
                        None
                    },
                },
            });
            HttpServer::new(move || {
                App::new()
                    .app_data(state.clone())
                    .route("/hello", web::get().to(|| async { "Hello World!" }))
                    .service(prometheus::prometheus_hook)
                    .service(greet)
                    .service(machine::list_machines)
                    .service(machine::add_machine)
                    .service(machine::remove_machine)
                    .service(service::add_service)
                    .service(service::remove_service)
                    .service(service::list_services)
            })
            .bind((server_opts.host, server_opts.port))?
            .run()
            .await.unwrap();
        },
        _ => error!("not implemented"),
    };
    Ok(())
}
