use serde::{Serialize, Deserialize};
use std::fs;
use std::error::Error;

// Flume API
static FLUME_API: &str = "https://api.flumewater.com/";

// Application configuration data
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AppConfig {
    credentials : Credential,
}

// Credentials for connecting to Flume service
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Credential {
    username: String,
    password: String,
    client_id: String,
    client_secret: String
}

// Login request
#[derive(Debug, PartialEq, Serialize)]
struct LoginPayload {
    grant_type: String,
    client_id: String,
    client_secret: String,
    username: String,
    password: String
}

// Cred to Login
// Create login payload from creds
fn cred_to_login(cred: &Credential) -> LoginPayload {
    return LoginPayload {
        grant_type: String::from("password"),
        client_id: cred.client_id.clone(),
        client_secret: cred.client_secret.clone(),
        username: cred.username.clone(),
        password: cred.password.clone()
    }
}

// Read configuration from file Yaml file in current directory
fn read_config() -> Result<AppConfig, Box<dyn Error>> {
    let cfgfile = "config.yml";
    let cfgstr = fs::read_to_string(cfgfile).expect("failed to read config");
    let cfgr : Result<AppConfig, serde_yaml::Error> = serde_yaml::from_str(&cfgstr);
    return Ok(cfgr?);
}

// Get an access/refresh token
fn get_access_token(cred: &Credential) {
    let login_payload = cred_to_login(cred);
    let url = format!("{}{}", FLUME_API, "oauth/token");
    println!("{}", url);
    let bodyres = reqwest::blocking::get(&url).expect("Req failed")
    .text().expect("convsion to text failed");
    println!("{}", bodyres);
//    headers = {'content-type': 'application/json'}
 //       flume_token_url = self.flume_api+ "oauth/token"
  //      response_json = requests.request("POST", flume_token_url, data=payload_str, headers=headers)
}

#[allow(unused_variables)]
fn main() {
    println!("Flume2Nats starting up...");
    println!("Reading config file");
    let cfg = read_config().expect("Config could not be read"); 
    //println!("{:?}",cfg);
    // Attempt to get access & refresh tokens
    get_access_token(&cfg.credentials);
}
