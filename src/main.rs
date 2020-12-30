use serde::{Serialize, Deserialize};
use std::fs;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use base64;

#[derive(Debug)]
pub enum FlumeError {
  MissingClaimError,
}

impl std::error::Error for FlumeError {}

impl fmt::Display for FlumeError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      FlumeError::MissingClaimError => write!(f, "Missing Required Claim in JWT Error"),
    }
  }
}

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
    data: Vec<AccessTokenData>,
}
// AccessToken data
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AccessTokenData {
    token_type: String,
    access_token: String,
    expires_in: i64,
    refresh_token: String,
}

#[derive(Debug, PartialEq)]
struct AccessToken {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}

// Flume User
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FlumeUser {
    email: String,
    id: String
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
fn get_access_token(cred: &Credential) -> Option<AccessToken> {
    let login_payload = cred_to_login(cred);
    // create json string
    let login_payload_str = serde_json::to_string(&login_payload).expect("Can't create payload json");
    let url = format!("{}{}", FLUME_API, "oauth/token");
    let client = reqwest::blocking::Client::new();
    let bodyres = client.post(&url).header("content-type", "application/json").body(login_payload_str).send().expect("Req failed")
        .text().expect("conversion to text failed");
    let oauth_reply : OauthReply = serde_json::from_str(&bodyres).expect("Could not deserialize token");
    // todo - assert we have a single result, and that the response was successful
    let tok = &oauth_reply.data[0];
    // Calculate expiration as epoch time
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("could not get epoch time").as_secs() as i64;
    let expire_at = now + tok.expires_in;
    Some(AccessToken {access_token: tok.access_token.clone(),
        expires_at: expire_at, refresh_token: tok.refresh_token.clone()})
}

// The user ID is built-in to the access token
fn get_user_id(tok: &AccessToken) -> Result<i64,Box<dyn Error>> {
    let t = &tok.access_token;
    // Break apart the JWT into header/claim/sig components 
    let components: Vec<&str> = t.split('.').collect();
    let claims_encoded = components[1];
    // base64-decode the token
    let decode = base64::decode(claims_encoded)?; //.expect("can't decode token"); 
    let decstr = String::from_utf8_lossy(&decode);
    // parse json
    let parsed : serde_json::Value = serde_json::from_str(&decstr)?;
    // Get the user_id number
    let uid = parsed["user_id"].as_i64().unwrap();
    //Err(Box::new(FlumeError::MissingClaimError)) 
    Ok(uid)
}

#[allow(unused_variables)]
fn main() {
    println!("Flume2Nats starting up...");
    println!("Reading config file");
    let cfg = read_config().expect("Config could not be read"); 
    //println!("{:?}",cfg);
    // Attempt to get access & refresh tokens
    let tok = get_access_token(&cfg.credentials).unwrap();
    println!("{:#?}", tok);
    // Now that we have a token, we can 
    // make an API call.
    
    // First step is to get the user ID, we need this later.
    let user_id = get_user_id(&tok).expect("could not get userid");
    println!("UserID: {}", user_id);


}
