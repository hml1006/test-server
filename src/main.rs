#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate yaml_rust;
use yaml_rust::{YamlLoader, Yaml};
use std::fs;
mod route;
mod statistic;
mod middleware;
mod types;

use std::error::Error;
use std::sync::Mutex;
use std::net::{IpAddr, SocketAddr};
use actix_web::{App, HttpServer};
use shellexpand;

use std::thread;
use std::time::Duration;

use route::{init_route_by_yaml};
use statistic::show_statistics;

/// version
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// application name
const NAME: &'static str = env!("CARGO_PKG_NAME");
/// application description
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
/// author
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
// keep_alive timeout: seconds
const KEEPALIVE: usize = 75;

const DEFAULT_STATS_REFRESH_INTERVAL: u64 = 1;

lazy_static! {
    // listen ip
    static ref LISTEN_IP: Mutex<IpAddr> = Mutex::new("0.0.0.0".parse::<IpAddr>().unwrap());
    // listen port
    static ref LISTEN_PORT: Mutex<u16> = Mutex::new(0);
    // statistics info refresh interval, s
    static ref STATS_REFRESH_INTERVAL: Mutex<u64> = Mutex::new(DEFAULT_STATS_REFRESH_INTERVAL);
    // yaml configuration
    static ref YAML_CONFIG: Mutex<Vec<Yaml>> = Mutex::new(Vec::new());
}

/// create statistics thread
fn create_statistics_thread() {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::new(*STATS_REFRESH_INTERVAL.lock().unwrap(), 0));
            show_statistics(*LISTEN_PORT.lock().unwrap());
        }
    });
}

/// init configuration
fn init_cfg() -> std::result::Result<(), Box<dyn Error>> {
    // build arguments parser
    let matches = clap_app!(myapp =>
        (name: NAME)
        (version: VERSION)
        (author: AUTHOR)
        (about: DESCRIPTION)
        (@arg host: --ip +takes_value "ip address to listen on")
        (@arg port: -p --port +takes_value "listening port number")
        (@arg interval: -i --interval +takes_value "refresh statistics information interval, default is 1 second")
        (@arg yaml: -y --yaml +takes_value "yaml configuration, configure urls and files mapping")
    ).get_matches();

    // parse or set default ipaddress
    *LISTEN_IP.lock().unwrap() = match matches.value_of("host").unwrap_or("0.0.0.0").parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            println!("parse ip failed: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // parse or set defalut port number
    *LISTEN_PORT.lock().unwrap() = match matches.value_of("port").unwrap_or("8088").parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            println!("parse port failed: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // parse statistics information interval
    *STATS_REFRESH_INTERVAL.lock().unwrap() = match matches.value_of("interval").unwrap_or(&DEFAULT_STATS_REFRESH_INTERVAL.to_string()).parse::<u64>() {
        Ok(interval) => interval,
        Err(e) => {
            println!("parse interval failed: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // get yaml configuration
    let yaml = matches.value_of("yaml");
    if yaml.is_none() {
        return Ok(());
    }
    // yaml file path
    let yaml = shellexpand::full(yaml.unwrap());
    let yaml = match yaml {
        Ok(yaml) => yaml.to_string(),
        Err(e) => {
            println!("expand yaml file path failed: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // read yaml file to string
    let yaml = match fs::read_to_string(yaml) {
        Ok(yaml) => yaml,
        Err(e) => {
            println!("read yaml file failed: {:?}", e);
            return Err(Box::new(e));
        }
    };

    parse_yaml(&yaml)?;

    Ok(())
}

// parse yaml
fn parse_yaml(yaml: &str) -> Result<(), Box<dyn Error>>{
    // parse yaml string
    let docs = match YamlLoader::load_from_str(yaml) {
        Ok(yaml) => yaml,
        Err(e) => {
            println!("parse yaml faile failed: {:?}", e);
            return Err(Box::new(e));
        }
    };
    println!("docs len: {}", docs.len());
    *YAML_CONFIG.lock().unwrap() = docs;
    Ok(())
}

fn main() {
    let init_ret = init_cfg();
    if let Err(_) = init_ret {
        println!("init failed!");
        return;
    }

    if YAML_CONFIG.lock().unwrap().len() == 0 {
        println!("yaml configuration empty!");
        return;
    }

    // init route information
    let doc = &(*YAML_CONFIG.lock().unwrap())[0];
    init_route_by_yaml(doc);

    // bind address
    let serv = HttpServer::new(|| {
        let app = App::new().wrap(middleware::ReqStat);
//        app.route();
        return app
    })
        .keep_alive(KEEPALIVE)
        .bind(SocketAddr::new(*LISTEN_IP.lock().unwrap(), *LISTEN_PORT.lock().unwrap()));

    let serv = match serv {
        Ok(serv) => serv,
        Err(e) => {
            println!("bind failed: {}", e);
            return;
        }
    };

    println!("listening on {}:{}", *LISTEN_IP.lock().unwrap(), *LISTEN_PORT.lock().unwrap());

    create_statistics_thread();

    // run http server
    match serv.run() {
        Err(e) => {
            println!("run server failed: {}", e);
        }
        _ => ()
    }
}