#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

mod get_serv;
mod statistic;
mod middleware;

use std::sync::Mutex;
use std::net::{IpAddr, SocketAddr};
use actix_web::{App, HttpServer};

use std::thread;
use std::time::Duration;


use get_serv::{ping, check, index};
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

const DEFAULT_STATS_REFRESH_INTERVAL: u64 = 5;

lazy_static! {
    static ref LISTEN_PORT: Mutex<u16> = Mutex::new(0);
    // statistics info refresh interval, s
    static ref STATS_REFRESH_INTERVAL: Mutex<u64> = Mutex::new(DEFAULT_STATS_REFRESH_INTERVAL);
}

/// set static variable listen_port
fn init_listen_port(port: u16) {
    *LISTEN_PORT.lock().unwrap() = port;
}

fn set_statistics_refresh_interval(seconds: u64) {
    *STATS_REFRESH_INTERVAL.lock().unwrap() = seconds;
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

fn main() {
    // build arguments parser
    let matches = clap_app!(myapp =>
        (name: NAME)
        (version: VERSION)
        (author: AUTHOR)
        (about: DESCRIPTION)
        (@arg host: -h --host +takes_value "ether address to listen on")
        (@arg port: -p --port +takes_value "listening port number")
        (@arg interval: -i --interval +takes_value "refresh statistics information interval, default 1 second")
    ).get_matches();

    // parse or set default ipaddress
    let ip = matches.value_of("host").unwrap_or("0.0.0.0").parse::<IpAddr>();
    let ip = match ip {
        Ok(ip) => ip,
        Err(e) => {
            println!("host parse error: {}", e);
            return;
        }
    };

    // parse or set defalut port number
    let port = matches.value_of("port").unwrap_or("8088").parse::<u16>();
    let port = match port {
        Ok(port) => port,
        Err(e) => {
            println!("port parse error: {}", e);
            return;
        }
    };
    init_listen_port(port);

    // parse statistics information interval
    let interval = matches.value_of("interval").unwrap_or(&DEFAULT_STATS_REFRESH_INTERVAL.to_string()).parse::<u64>();
    let interval = match interval {
        Ok(interval) => interval,
        Err(e) => {
            println!("interval parse error: {}", e);
            return;
        }
    };
    set_statistics_refresh_interval(interval);

    // bind address
    let serv = HttpServer::new(|| {
        App::new()
            .wrap(middleware::ReqStat)
            .service(index)
            .service(ping)
            .service(check)
    })
        .keep_alive(KEEPALIVE)
        .bind(SocketAddr::new(ip, port));

    let serv = match serv {
        Ok(serv) => serv,
        Err(e) => {
            println!("bind failed: {}", e);
            return;
        }
    };

    println!("listening on {}:{}", ip, port);

    create_statistics_thread();

    // run http server
    match serv.run() {
        Err(e) => {
            println!("run server failed: {}", e);
        }
        _ => ()
    }
}