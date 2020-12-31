use serde::{Serialize, Deserialize};
use std::fs;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use base64;
use chrono::{DateTime, Local, Duration};

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

// Usage Sample
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct UsageSample {
    datetime: String,
    gallons: f64,
}

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

// For now, we just return the first readable device
// TODO: use the location["tz"] field from the data object to get the named timezone
// ("America/Los_Angeles")
fn get_devices(tok: &AccessToken, userid: &i64) -> Result<String, Box<dyn Error>> {
    // URL is flume api /users/{userid}/devices
    let url = format!("{}users/{}/devices", FLUME_API, userid);
    let client = reqwest::blocking::Client::new();
    let bodyres = client.get(&url).header("Authorization", format!("Bearer {}",&tok.access_token))
        .query(&[("user", "false"), ("location", "true")]).send().expect("Req failed")
        .text().expect("conversion to text failed");
    // "success" should be True
    // "data" should contain an array of devices
    // we want the device with "type": 2 (reader)
    // and then we want its "id"
    let parsed : serde_json::Value = serde_json::from_str(&bodyres)?;
    let data = &parsed["data"].as_array().unwrap();
    // filter out non-type-2 devices 
    let filtered : Vec<&serde_json::Value>= data.iter().filter(|dev| dev["type"].as_i64().unwrap() == 2).collect();
    // TODO: check that we have one item
    let first_dev = filtered[0];
    let devid = first_dev["id"].as_str().unwrap();  
    return Ok(devid.to_string());
}

//fn to_usage(api_res: serde_json:Value) {

//}

fn get_current_usage(tok: &AccessToken, userid: &i64, deviceid: &String) {
    println!("Getting current usage");
    // get minute-by-minute usage for past 10 minutes 
    // just assume that our timezone matches up with the device timezone for now. 
    let url = format!("{}users/{}/devices/{}/query", FLUME_API, userid, deviceid); 
    let client = reqwest::blocking::Client::new();
//    let current_time = SystemTime::now();
 //   let ten_min_ago = current_time - Duration::from_secs(60*10);
  //  println!("{:#?} -> {:#?}", ten_min_ago, current_time);
    let now: DateTime<Local> = Local::now();
    let ten_min_ago = now - Duration::minutes(5);

    println!("{:#?} -> {:#?}", ten_min_ago, now);
    // Create ISO formatted date
    let now_str = now.format("%Y-%m-%d %H:%M:%S");
    let ten_min_ago_str = ten_min_ago.format("%Y-%m-%d %H:%M:%S");
    let query = format!(r#"
    {{
      "queries": [
        {{
          "bucket": "MIN",
          "since_datetime": "{}",
          "until_datetime": "{}",
          "request_id": "req-id"
        }}
      ]
    }}
    "#, ten_min_ago_str, now_str);
    //println!("{}", query);
//    let bodyres = client.post(&url).header("Authorization", format!("Bearer {}",&tok.access_token))
//       .body(query).send().expect("Req failed")
//        .text().expect("conversion to text failed");
    let bodyres = client.post(&url).header("Authorization", format!("Bearer {}",&tok.access_token))
        .header("content-type", "application/json")
        .body(query).send().expect("req failed").text().expect("no response body");

    //println!("{:#?}", bodyres);
    // parse last 60 minutes 
    let parsed : serde_json::Value = serde_json::from_str(&bodyres).expect("cannot parse json");
    //println!("{:#?}", parsed);
    // Create UsageSamples with datetime and gallons
    let data = &parsed["data"];
    //println!("{:#?}", data);
    let qdata = &data[0]["req-id"].as_array().unwrap();
    println!("{:#?}", qdata);
    let usage : Vec<_> = qdata.iter().map(|x| UsageSample{gallons: x["value"].as_f64().unwrap(), datetime: x["datetime"].as_str().unwrap().to_string()}).collect();
    println!("{:#?}", usage);
//    let samples = qdata.as_array().map(|x| x["value"].as_f64().unwrap()).collect();

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

    // Get devices
    let devid = get_devices(&tok, &user_id).expect("device not found");
    println!("DevID: {}", devid);

    // Now with the user and device ID, we can query for usage.
    get_current_usage(&tok, &user_id, &devid);

    println!("DevID: {}", devid);
}
