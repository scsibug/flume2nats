use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AppConfig {
    credentials : Credential,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Credential {
    username: String,
    password: String,
    client_id: String,
    client_secret: String
}

fn read_config() -> Result<AppConfig, serde_yaml::Error> {
    let cfgfile = "config.yml";
    let cfgstr = fs::read_to_string(cfgfile).expect("failed to read config");
    let cfgr : Result<AppConfig, serde_yaml::Error> = serde_yaml::from_str(&cfgstr);
    return cfgr;
}

#[allow(unused_variables)]
fn main() {
    println!("Flume2Nats starting up...");
    println!("Reading config file");
    let cfgr = read_config(); 
    if let Ok(cfg) = cfgr {
        println!("{:?}",cfg);
    } else {
        println!("Failed to read app settings");
    }
}
