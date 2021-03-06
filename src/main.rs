#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate dashmap;
extern crate hyper;
extern crate tokio;
extern crate yaml_rust;
extern crate chrono;
extern crate netstat;

mod types;

use console::{Term, Color, style};
use dashmap::DashMap;
use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use itertools::Itertools;
use shellexpand;
use std::boxed::Box;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::net::IpAddr;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::vec::Vec;
use thread_id;
use yaml_rust::yaml::Yaml::{Array, Hash};
use yaml_rust::{Yaml, YamlLoader};
use chrono::prelude::*;
use netstat::*;
use std::process;

use crate::types::mime_types::MimeType;
use crate::types::route::{Content, RouteInfo};

/// version
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// application name
const NAME: &'static str = env!("CARGO_PKG_NAME");
/// application description
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
/// author
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

// keys
const KEY_IP: &'static str = "ip";
const KEY_PORT: &'static str = "port";
const KEY_INTERNAL: &'static str = "internal";

// if a file size small then MAX_FILE_CACHE_LENGTH, then this file will be cached
const MAX_FILE_CACHE_LENGTH: u64 = 512 * 1024;

// default statistics information refresh time
const DEFAULT_STATS_REFRESH_INTERVAL: u64 = 1;

//default listen port
const DEFAULT_LISTEN_PORT: u16 = 8088;

lazy_static! {
    //parameters from command line
    static ref CONFIGURATION: DashMap<&'static str, String> = DashMap::new();
    // yaml configuration
    static ref YAML_CONFIG: Mutex<Vec<Yaml>> = Mutex::new(Vec::new());
    // routes configuration
    static ref ROUTES: DashMap<String, RouteInfo> = DashMap::new();
    // file cache
    static ref FILE_CACHE: DashMap<String, Arc<Box<Vec<u8>>>> = DashMap::new();
    // statistics, structure
    // thread_id 1 => status code 200 => 20
    //             => status code 404 => 32
    // thread_id 2 => status code 200 => 11
    //             => status code 403 => 22
    static ref STATISTICS: DashMap<usize, DashMap<u16, Box<u64>>> = DashMap::new();
    // total connections, this variable stores all connections number that has been received from program start to now
    static ref TOTAL_CONNECTIONS: RwLock<u64> = RwLock::new(0);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_BACKTRACE", "full");
    let ret = parse_args();
    if let Err(_) = ret {
        println!("init failed!");
        return Ok(());
    }

    // init route information
    if YAML_CONFIG.lock().unwrap().len() > 0 {
        let yaml = YAML_CONFIG.lock().unwrap();
        let doc = yaml.get(0);
        match doc {
            Some(doc) => {
                init_route_by_yaml(doc);
            }
            None => {
                println!("yaml error");
                return Ok(())
            }
        }
    }

    let addr = {
        format!(
            "{}:{}",
            CONFIGURATION.get(KEY_IP).unwrap().value(),
            CONFIGURATION.get(KEY_PORT).unwrap().value()
        )
    };
    println!("{}", style(format!("listening on {}", addr)).bold().italic().yellow());
    let addr = addr.parse().unwrap();

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| {
        async {
            inc_connections();
            Ok::<_, Infallible>(service_fn(response))
        }
    });

    create_stat_thread();

    // Then bind and serve...
    // wait for web service start
    let server = Server::bind(&addr).tcp_keepalive(Some(Duration::from_secs(60))).http1_keepalive(true).serve(make_service);
    server.await?;

    Ok(())
}

/// increase the response number by thread id and status code
fn inc_response(thread_id: usize, status_code: u16) {
    let thread_statistics = STATISTICS.get(&thread_id);
    match thread_statistics {
        Some(thread_statistics) => {
            let thread_statistics = thread_statistics.value();
            let http_statistics = thread_statistics.get_mut(&status_code);
            match http_statistics {
                Some(mut http_statistics) => {
                    let ptr = http_statistics.value_mut();
                    let count = **ptr;
                    **ptr = count + 1;
                }
                None => {
                    thread_statistics.insert(status_code.clone(), Box::new(1u64));
                }
            }
        }
        None => {
            let http_statistics = DashMap::new();
            http_statistics.insert(status_code.clone(), Box::new(1u64));
            STATISTICS.insert(thread_id, http_statistics);
        }
    }
}

/// if a new connection comming, increase the global count
fn inc_connections() {
    *TOTAL_CONNECTIONS.write().unwrap() += 1;
}

/// get total connections number
fn get_total_connections() -> u64 {
    *TOTAL_CONNECTIONS.read().unwrap()
}

/// get response statistics
/// status code -> count
fn get_response_statistic() -> Box<HashMap<u16, u64>> {
    let mut statistic = Box::new(HashMap::new());
    STATISTICS.iter().for_each(|thread_statistic| {
        thread_statistic.iter().for_each(|status_statistic| {
            let code = *status_statistic.key();
            let current_count = **status_statistic.value();
            let status = statistic.get(&code);
            match status {
                Some(count) => {
                    let total = current_count + count;
                    statistic.insert(code, total);
                }
                None => {
                    statistic.insert(code, current_count);
                }
            }
        });
    });
    statistic
}

/// get all connections by listening port
fn get_connections_info_by_listen_port(listen_port: u16) -> Result<Vec<SocketInfo>, Error> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    let sockets_info = get_sockets_info(af_flags, proto_flags)?;
    let process_id = process::id();
    let sockets_info = sockets_info.into_iter()
        .filter(|si| {
            match &si.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_si) => {
                    tcp_si.local_port == listen_port && si.associated_pids.contains(&process_id)
                }
                _ => false
            }
        })
        .collect::<Vec<SocketInfo>>();
    Ok(sockets_info)
}

async fn response(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let url = req.uri().path().to_string();
    let thread_id: usize = thread_id::get();
    match ROUTES.get(&url) {
        Some(route) => {
            let route = route.value();
            if route.method == req.method() {
                let builder = hyper::Response::builder();
                let builder = builder.status(route.status_code);
                let mut builder = builder.header("Content-Type", route.mime_type.to_string());
                let headers = builder.headers_mut().unwrap();
                route.headers.iter().for_each(|(key, value)| {
                    headers.insert(key, value.clone());
                });
                match &route.body {
                    Content::Cache => {
                        let content = FILE_CACHE.get(&url);
                        match content {
                            Some(content) => {
                                let len = content.len();
                                let raw = content.as_ptr();
                                unsafe {
                                    inc_response(thread_id, route.status_code.as_u16());
                                    Ok(builder
                                        .body(Body::from(std::slice::from_raw_parts(raw, len)))
                                        .unwrap())
                                }
                            }
                            None => {
                                println!("url: {} cache not found", &url);
                                inc_response(thread_id, StatusCode::NOT_FOUND.as_u16());
                                Ok(builder
                                    .status(StatusCode::NOT_FOUND)
                                    .body(Body::from("not found"))
                                    .unwrap())
                            }
                        }
                    }
                    Content::Content(content) => {
                        inc_response(thread_id, route.status_code.as_u16());
                        Ok(builder.body(Body::from(content.clone())).unwrap())
                    }
                    Content::File(file) => {
                        inc_response(thread_id, route.status_code.as_u16());
                        Ok(builder.body(Body::from(file.clone())).unwrap())
                    }
                }
            } else {
                inc_response(thread_id, StatusCode::METHOD_NOT_ALLOWED.as_u16());
                Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Body::from("method for this request is not implemented"))
                    .unwrap())
            }
        }
        None => {
            inc_response(thread_id, StatusCode::NOT_FOUND.as_u16());
            // println!("url: {} not found", url);
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap())
        }
    }
}

fn write_term(term: &Term, msg: &str, term_line_num: usize) -> usize {
    match term.write_line(msg) {
        Ok(_) => term_line_num + 1,
        Err(e) => {
            println!("write termimal failed: {}", e);
            term_line_num.clone()
        }
    }
}

fn get_netstat_info() -> (usize, usize, usize) {
    let listen_port = CONFIGURATION.get(KEY_PORT).unwrap().value().parse::<u16>().unwrap_or(DEFAULT_LISTEN_PORT);
    let sockets_info = match get_connections_info_by_listen_port(listen_port) {
        Ok(sockets_info) => sockets_info,
        Err(e) => {
            println!("Error: get sockets info failed: {:?}", e);
            Vec::new()
        }
    };

    // syn-recvd
    let connecting_num = sockets_info.iter().filter(|si| {
        match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                match tcp_si.state {
                    TcpState::SynReceived => true,
                    _ => false
                }
            }
            _ => false
        }
    }).count();

    // is closing
    let closing_num = sockets_info.iter().filter(|si| {
        match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                match tcp_si.state {
                    TcpState::FinWait1 | TcpState::FinWait2 | TcpState::CloseWait | TcpState::Closing |
                    TcpState::LastAck | TcpState::TimeWait => true,
                    _ => false
                }
            }
            _ => false
        }
    }).count();

    // established
    let established_num = sockets_info.iter().filter(|si| {
        match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                match tcp_si.state {
                    TcpState::Established => true,
                    _ => false
                }
            }
            _ => false
        }
    }).count();

    (connecting_num, closing_num, established_num)
}

/// create statistics thread
fn create_stat_thread() {
    thread::spawn(move || {
        // terminal to show statistics
        let term = console::Term::stderr();
        term.hide_cursor().unwrap();
        // terminal line number
        let mut term_line_num = 0;
        loop {
            // sleep
            let durection = Duration::from_secs(
                CONFIGURATION
                    .get(KEY_INTERNAL)
                    .unwrap()
                    .value()
                    .parse()
                    .unwrap(),
            );
            thread::sleep(durection);

            let (connecting, closing, established) = get_netstat_info();

            // clear termimal output
            match term.clear_last_lines(term_line_num) {
                Ok(_) => {
                    term_line_num = 0;

                    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    term_line_num = write_term(&term, &format!("{} {} {}", style("***************").bold().cyan(), style(current_time).bold().green(),
                        style("***************").bold().cyan()), term_line_num.clone());
                    term_line_num = write_term(
                        &term,
                        &format!("[{}] {}", style("Connections from start").bold().italic().yellow().bg(Color::Black), style(get_total_connections()).bg(Color::Black).white().bold()),
                        term_line_num.clone(),
                    );
                    term_line_num = write_term(
                        &term,
                        &format!("[{}] {}", style("Connecting").bold().italic().yellow().bg(Color::Black), style(connecting).bg(Color::Black).white().bold()),
                        term_line_num.clone(),
                    );
                    term_line_num = write_term(
                        &term,
                        &format!("[{}] {}", style("Closing").bold().italic().yellow().bg(Color::Black), style(closing).bg(Color::Black).white().bold()),
                        term_line_num.clone(),
                    );
                    term_line_num = write_term(
                        &term,
                        &format!("[{}] {}", style("Established").bold().italic().yellow().bg(Color::Black), style(established).bg(Color::Black).white().bold()),
                        term_line_num.clone(),
                    );
                    term_line_num =
                        write_term(&term, &format!("{}", style("-----------------------------------").green()), term_line_num.clone());
                    //get response statistics
                    let iter = get_response_statistic().into_iter();
                    let resp_status_statistic: Vec<_> = iter.collect();

                    // response statistics start from third bar
                    for (_, element) in resp_status_statistic
                        .into_iter()
                        .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                        .into_iter()
                        .enumerate()
                    {
                        let (code, count) = element;
                        term_line_num = write_term(&term, &format!("[{}] {}", style(code).bold().italic().yellow().bg(Color::Black), 
                            style(count).bg(Color::Black).white().bold()), term_line_num.clone());
                    }
                }
                Err(e) => {
                    println!("clear term failed: {}", e);
                    continue;
                }
            }
        }
    });
}

/// init configuration
fn parse_args() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    let ip_str = matches.value_of("host").unwrap_or("0.0.0.0");
    match ip_str.parse::<IpAddr>() {
        Ok(_) => {}
        Err(e) => {
            println!("parse ip failed: {:?}", e);
            return Err(Box::new(e));
        }
    };
    CONFIGURATION.insert(KEY_IP, ip_str.to_string());

    // parse or set defalut port number
    let port = match matches.value_of("port").unwrap_or(&DEFAULT_LISTEN_PORT.to_string()).parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            println!("parse port failed: {:?}", e);
            return Err(Box::new(e));
        }
    };
    CONFIGURATION.insert(KEY_PORT, port.to_string());

    // parse statistics information interval
    let interval = match matches
        .value_of("interval")
        .unwrap_or(&DEFAULT_STATS_REFRESH_INTERVAL.to_string())
        .parse::<u64>()
    {
        Ok(interval) => interval,
        Err(e) => {
            println!("parse interval failed: {:?}", e);
            return Err(Box::new(e));
        }
    };
    CONFIGURATION.insert(KEY_INTERNAL, interval.to_string());

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

    println!("yaml path: {}", yaml);

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
fn parse_yaml(yaml: &str) -> Result<(), Box<dyn std::error::Error>> {
    // parse yaml string
    let docs = match YamlLoader::load_from_str(yaml) {
        Ok(yaml) => yaml,
        Err(e) => {
            println!("parse yaml faile failed: {:?}", e);
            return Err(Box::new(e));
        }
    };
    *YAML_CONFIG.lock().unwrap() = docs;
    Ok(())
}

// init route from yaml
fn init_route_by_yaml(yaml: &Yaml) {
    let yaml = match yaml {
        Hash(yaml) => yaml,
        _ => return,
    };

    for (key, value) in yaml.iter() {
        // get array
        let value = match value {
            Array(yaml) => yaml,
            _ => {
                println!("the method's elements should be an array");
                continue;
            }
        };

        //  build methods routes
        let key = key.as_str().unwrap_or("GET").to_uppercase();
        let method = match Method::from_str(&key) {
            Ok(method) => method,
            Err(e) => {
                println!("method error: {}", e);
                continue;
            }
        };

        // initialize keys
        let url_key = yaml_rust::Yaml::String("url".to_string());
        let file_key = yaml_rust::Yaml::String("file".to_string());
        let headers_key = yaml_rust::Yaml::String("headers".to_string());
        //        let status_code_key = yaml_rust::Yaml::String("status_code".to_string());

        // filter from array that has url filed.
        let value = value
            .iter()
            .filter(|element| match element {
                Hash(element) => element.contains_key(&url_key),
                _ => {
                    println!("request configuration should be hash type: {:?}", element);
                    false
                }
            })
            .collect::<Vec<&Yaml>>();

        // traverse all requests configuration
        for req in value.into_iter() {
            match req {
                Hash(element) => {
                    // get url
                    let url = element.get(&url_key).unwrap();
                    let url = match url {
                        yaml_rust::yaml::Yaml::String(url) => url.clone(),
                        _ => {
                            println!("url not string: {:?}", url);
                            continue;
                        }
                    };

                    // mime type, body and status code
                    let (mime_type, body, status_code) =
                        match parse_mime_and_body(&req, &file_key, url.clone()) {
                            Ok(value) => value,
                            Err(e) => {
                                println!("error occurred while parsing mime and body: {}", e);
                                continue;
                            }
                        };

                    // parse headers
                    let headers = element.get(&headers_key);
                    let headers: HeaderMap<HeaderValue> = if headers.is_none() {
                        Default::default()
                    } else {
                        parse_headers(headers.unwrap())
                    };

                    println!("insert url: {}", &url);
                    // add route
                    ROUTES.insert(
                        url.clone(),
                        RouteInfo {
                            url,
                            method: method.clone(),
                            status_code,
                            mime_type,
                            headers,
                            body,
                        },
                    );
                }
                _ => {
                    println!("not hash element");
                }
            }
        }
    }
}

fn parse_headers(yaml: &Yaml) -> HeaderMap {
    let headers = match yaml {
        Hash(headers) => headers,
        _ => {
            println!("header type error: {:?}", yaml);
            return Default::default();
        }
    };

    let mut header_map = HeaderMap::new();

    for (key, value) in headers.iter() {
        match key {
            yaml_rust::yaml::Yaml::String(key) => match value {
                yaml_rust::yaml::Yaml::String(value) => {
                    let key = match HeaderName::from_str(key.as_str()) {
                        Ok(key) => key,
                        Err(e) => {
                            println!("error header name: {}", e);
                            continue;
                        }
                    };
                    let value = match HeaderValue::from_str(value.as_str()) {
                        Ok(value) => value,
                        Err(e) => {
                            println!("error header value: {}", e);
                            continue;
                        }
                    };
                    header_map.insert(key, value);
                }
                _ => {
                    println!("value type error: {:?}", value);
                    continue;
                }
            },
            _ => {
                println!("key type error: {:?}", key);
                continue;
            }
        }
    }

    header_map
}

fn parse_mime_and_body(
    yaml: &Yaml,
    file_key: &yaml_rust::yaml::Yaml,
    url: String,
) -> Result<(MimeType, Content, StatusCode), Box<dyn std::error::Error>> {
    let element = match yaml {
        Hash(yaml) => yaml,
        _ => {
            return Err(String::from("yaml type is not hash").into());
        }
    };
    // get file path and convert to body
    let mime_type = MimeType::ApplicationOctetStream;

    // file filed not found, return
    let file = element.get(file_key);
    if file.is_none() {
        return Ok((
            MimeType::TextPlain,
            Content::Content("not found file path field".to_string()),
            StatusCode::NOT_FOUND,
        ));
    };

    match file.unwrap() {
        yaml_rust::yaml::Yaml::String(path) => {
            let path = path.to_string();
            let full_path = shellexpand::full(&path);
            if full_path.is_ok() {
                let full_path = full_path.unwrap().to_string();
                let abs_path = Path::new(&full_path);
                // not file or no permmision to access, return
                if !abs_path.is_file() {
                    println!("file error: {:?}", abs_path.as_os_str());
                    return Ok((
                        MimeType::TextPlain,
                        Content::Content(format!("not a file: {:?}", abs_path).to_string()),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
                // check file extension
                let extension = abs_path.extension();
                if extension.is_none() {
                    return Ok((mime_type, Content::File(full_path), StatusCode::OK));
                } else {
                    match extension.unwrap().to_str() {
                        Some(extension) => {
                            let mime_type = MimeType::from_str(extension)
                                .unwrap_or(MimeType::ApplicationOctetStream);
                            if mime_type.is_text() {
                                let ref meta = fs::metadata(abs_path);
                                if meta.is_err() {
                                    println!(
                                        "get file metadata failed: {:?}",
                                        meta.as_ref().err().unwrap()
                                    );
                                    return Ok((
                                        MimeType::TextPlain,
                                        Content::Content(
                                            format!(
                                                "get file metadata failed: {:?} => {:?}",
                                                abs_path,
                                                meta.as_ref().err().unwrap()
                                            )
                                            .to_string(),
                                        ),
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    ));
                                }
                                let file_length = meta.as_ref().unwrap().len();
                                if file_length <= MAX_FILE_CACHE_LENGTH {
                                    let file = File::open(&full_path);
                                    let mut buffer: Box<Vec<u8>> = Box::new(Vec::new());
                                    let mut file = match file {
                                        Ok(file) => file,
                                        Err(e) => {
                                            println!("open file failed: {:?}", e);
                                            return Ok((
                                                MimeType::TextPlain,
                                                Content::Content(
                                                    format!(
                                                        "open file failed: {:?} => {:?}",
                                                        abs_path, e
                                                    )
                                                    .to_string(),
                                                ),
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                            ));
                                        }
                                    };
                                    match file.read_to_end(buffer.as_mut()) {
                                        Ok(_) => {
                                            FILE_CACHE.insert(url, Arc::new(buffer));
                                            return Ok((mime_type, Content::Cache, StatusCode::OK));
                                        }
                                        Err(e) => {
                                            println!("read file failed: {:?} => {:?}", e, abs_path);
                                            return Ok((
                                                MimeType::TextPlain,
                                                Content::Content(
                                                    format!(
                                                        "read file failed: {:?} => {:?}",
                                                        e, abs_path
                                                    )
                                                    .to_string(),
                                                ),
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                            ));
                                        }
                                    }
                                } else {
                                    return Ok((
                                        mime_type,
                                        Content::File(abs_path.to_str().unwrap().to_string()),
                                        StatusCode::OK,
                                    ));
                                }
                            } else {
                                return Ok((
                                    MimeType::ApplicationOctetStream,
                                    Content::File(full_path),
                                    StatusCode::OK,
                                ));
                            }
                        }
                        _ => Err(String::from(format!(
                            "extension to string failed: {:?}",
                            abs_path
                        ))
                        .into()),
                    }
                }
            } else {
                Err(String::from(format!("path expend failed: {:?}", path)).into())
            }
        }
        _ => Err(String::from(format!("file path type error: {:?}", file)).into()),
    }
}

#[allow(dead_code)]
fn parse_status_code(yaml: &Yaml, status_code_key: &yaml_rust::yaml::Yaml) -> StatusCode {
    let element = match yaml {
        Hash(yaml) => yaml,
        _ => {
            panic!("yaml type is not hash: {:?}", yaml);
        }
    };
    element.get(status_code_key).map_or_else(
        || StatusCode::from_u16(200).unwrap(),
        |value| {
            let status = match value {
                yaml_rust::yaml::Yaml::String(code) => match StatusCode::from_str(code.as_str()) {
                    Ok(status) => Some(status),
                    Err(e) => {
                        println!("parse status code failed: {}", e);
                        None
                    }
                },
                _ => {
                    println!("unknown status code: {:?}", value);
                    None
                }
            };
            if status.is_none() {
                println!("use default status code 200");
                return StatusCode::from_u16(200).unwrap();
            }
            status.unwrap()
        },
    )
}
