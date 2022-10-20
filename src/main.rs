use actix_web::{
    web::{Data},
};
use clap::{arg, ArgMatches, Command};
use env_logger::{Builder, Target};
use log::{debug, error, info, warn};

use crate::state::AppState;
mod prometheus;
mod state;

fn greet(req: http::Request) -> http::Response {
    http::Response {
        status: 200,
        body: format!("Hello!"),
    }
}

fn hook(req: http::Request) -> http::Response {
    info!("Hook called");
    info!("{:?}", req.body);
    http::Response {
        status: 200,
        body: format!("Hello!"),
    }
}

fn cli() -> Command {
    let port_arg = arg!(-p - -port <PORT> "Specify a port to listen")
        .value_parser(clap::value_parser!(u16).range(3000..))
        .required(false);
    let host_arg = arg!(-H - -host <HOST> "Specify a host to listen").required(false);
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Start a node")
                .arg(arg!(--"wechat-robot" <WECHAT_ROBOT> "Hook URL of the wechat robot"))
                .arg(&port_arg)
                .arg(&host_arg),
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
    ServerOpts { host: h, port: p }
}

fn main() -> std::io::Result<()> {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();
    debug!("Starting environment logger");
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let server_opts = get_server_opts(sub_matches);
            info!(
                "Start listen on http://{}:{}",
                server_opts.host, server_opts.port
            );
            // Create state
            let state = Data::new(AppState {
                // sender: sender.clone(),
                wechat_robot: match sub_matches.get_one::<String>("wechat-robot") {
                    Some(bot) => Some(bot.clone()),
                    None => {
                        warn!("You not specified a wechat robot");
                        None
                    }
                },
            });
            let _ = http::WebServer::new()
                .route("/hello", greet)
                .route("/prometheus/hook", hook)
                .run();
        }
        _ => error!("not implemented"),
    };
    Ok(())
}
