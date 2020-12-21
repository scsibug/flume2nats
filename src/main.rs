use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AppConfig {
    apikey: String,
}

#[allow(unused_variables)]
fn main() {
    println!("Flume2Nats starting up...");
    println!("Reading config file");
    // Config should be located in current directory, named config.yml
    // a config file looks like:
    // Read file into string
    let config_from_file : Result<String, std::io::Error> = fs::read_to_string("config.yml"); 
    let configstr = "---\napikey: test";
    let cfg : Result<AppConfig,serde_yaml::Error> = serde_yaml::from_str(&configstr);
    println!("{:?}",cfg);
}
