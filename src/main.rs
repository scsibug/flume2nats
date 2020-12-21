use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AppConfig {
    apikey: String,
}

fn main() {
    println!("Flume2Nats starting up...");
    println!("Reading config file");
    // Config should be located in current directory, named config.yml
    // a config file looks like:
    let configstr = "---\napikey: test";
    let cfg : Result<AppConfig,serde_yaml::Error> = serde_yaml::from_str(&configstr);
    println!("{:?}",cfg);
}
