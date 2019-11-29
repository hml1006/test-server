extern crate lazy_static;
extern crate yaml_rust;

use yaml_rust::Yaml;
use actix_web::{HttpResponse, Responder};
use std::sync::RwLock;
use std::collections::HashMap;
use std::vec::Vec;
use yaml_rust::yaml::Yaml::{Hash, Array, String};
use std::fs;

use super::types::mime_types::*;

enum Method {
    Get,
    Post,
    Put,
    Delete,
}

enum FileInfo {
    Path(std::string::String),
    Cache(std::string::String),
}

pub struct RouteInfo {
    url: std::string::String,
    method: Method,
    file: FileInfo,
    mime: std::string::String
}

lazy_static! {
    static ref ROUTES: RwLock<HashMap<std::string::String, RouteInfo>> = RwLock::new(HashMap::new());
}

fn init_get_route(yaml: &Vec<&Yaml>) {
    yaml.iter().for_each(|item| {
        let item = match *item {
            Hash(item) => item,
            _ => return
        };
        let url_key = String("url".to_string());
        let file_key = String("file".to_string());
        let url = item[&url_key].as_str().unwrap();
        let file = item[&file_key].as_str().unwrap();
        ROUTES.write().unwrap().insert(url.to_string(), RouteInfo {
            url: url.to_string(),
            method: Method::Get,
            file: FileInfo::Path(file.to_string())
        });
    });
}

fn init_post_route(yaml: &Vec<&Yaml>) {}

fn init_put_route(yaml: &Vec<&Yaml>) {}

fn init_delete_route(yaml: &Vec<&Yaml>) {}

pub fn init_route_by_yaml(yaml: &Yaml) {
    let yaml = match yaml {
        Hash(yaml) => yaml,
        _ => return
    };

    for (key, value) in yaml.iter() {
        // get array
        let value = match value {
            Array(yaml) => yaml,
            _ => return
        };

        // filter from array that has url and file filed.
        let url_key = String("url".to_string());
        let file_key = String("file".to_string());
        let value = value.iter().filter(|element| {
            match element {
                Hash(element) => {
                    element.contains_key(&url_key) && element.contains_key(&file_key)
                },
                _ => {
                    println!("error type: {:?}", element);
                    false
                }
            }
        }).collect::<Vec<&Yaml>>();

        //  build methods routes
        if key.as_str().unwrap_or("") == "get" {
            init_get_route(&value);
        } else if key.as_str().unwrap_or("") == "post" {
            init_post_route(&value);
        } else if key.as_str().unwrap_or("") == "put" {
            init_put_route(&value);
        } else if key.as_str().unwrap_or("") == "delete" {
            init_delete_route(&value);
        }
    }
}