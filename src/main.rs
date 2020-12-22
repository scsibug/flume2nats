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

// OauthReply
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OauthReply {
    success: bool,
    data: Vec<AccessToken>,
}
// AccessToken
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AccessToken {
    token_type: String,
    access_token: String,
    expires_in: i64,
    refresh_token: String
}

// Create login payload struct from creds struct
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

// Get an access/refresh token, by making an HTTP call to Flume
fn get_access_token(cred: &Credential) {
    let login_payload = cred_to_login(cred);
    // create json string
    let login_payload_str = serde_json::to_string(&login_payload).expect("Can't create payload json");
    println!("{}",login_payload_str);
    let url = format!("{}{}", FLUME_API, "oauth/token");
    println!("{}", url);
    let client = reqwest::blocking::Client::new();
    let bodyres = client.post(&url).header("content-type", "application/json").body(login_payload_str).send().expect("Req failed")
    .text().expect("convsion to text failed");
    println!("{}", bodyres);
    let oauth_reply : OauthReply = serde_json::from_str(&bodyres).expect("Could not deserialize token");
    println!("{:#?}", oauth_reply);
    // body res is a JSON object with a data field that we want to serialize into an AccessToken
    // object.
    // The first elem of the data field from the OAuth reply has what we need.
    // todo - assert we have a single result, and that the response was successful
    let tok = &oauth_reply.data[0];
    println!("{:#?}", tok.access_token);
    println!("{:#?}", tok.refresh_token);
    println!("{:#?}", tok.expires_in);
    // What is the actual expiration time?
    // Easier; lets just 
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
