use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spinners::{Spinner, Spinners};
use std::thread;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
    verification_uri_complete: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Identity {
    connection: String,
    isSocial: bool,
    provider: String,
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeoIp {
    city_name: String,
    continent_code: String,
    country_code: String,
    country_code3: String,
    country_name: String,
    latitude: f64,
    longitude: f64,
    time_zone: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserMetadata {
    geoip: Option<GeoIp>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    created_at: String,
    email: Option<String>,
    email_verified: Option<Value>,
    family_name: Option<String>,
    given_name: Option<String>,
    identities: Vec<Identity>,
    last_ip: Option<String>,
    last_login: Option<String>,
    locale: Option<String>,
    logins_count: Option<u32>,
    name: String,
    nickname: String,
    picture: String,
    updated_at: String,
    user_id: String,
    user_metadata: Option<UserMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

impl AccessTokenResponse {
    async fn get_access_token(device_code: String) -> Result<String, reqwest::Error> {
        async fn poll_fn(device_code: &String) -> Result<String, reqwest::Error> {
            let client = reqwest::Client::new();
            let request = client
            .post("https://auth.rosnovsky.us/oauth/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "client_id=59WtMZZWhn2sXvNluQUB45qXAg7NHsYt&grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code={}",
                device_code
            ));
            let response = request.send().await.unwrap();
            // get the response body in json format
            let body = response.text().await.unwrap();
            // deserialize the json body into the AccessTokenResponse struct
            Ok(body)
        }
        let mut access_token = poll_fn(&device_code).await.unwrap();
        thread::sleep(std::time::Duration::from_secs(5));
        for _ in 1..=10 {
            access_token = poll_fn(&device_code).await.unwrap();
            if !access_token.contains("access_token") {
                println!("Pending auth: {}", access_token);
                thread::sleep(std::time::Duration::from_secs(5));
            } else {
                println!("AC has access_token: {}", access_token);
                break;
            }
        }

        match access_token.contains("access_token") {
            true => Ok(access_token),
            false => Ok("Error".to_string()),
        }
    }
}

impl DeviceCodeResponse {
    async fn get_device_code() -> Result<String, reqwest::Error> {
        // Create a new client
        let client = reqwest::Client::new();
        // create a new post request to https://auth.rosnovsky.us/oauth/token with the client_credentials grant type
        let request = client
        .post("https://auth.rosnovsky.us/oauth/device/code")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("client_id=59WtMZZWhn2sXvNluQUB45qXAg7NHsYt&scope=read:users&audience=https://rosnovsky.auth0.com/api/v2/");

        //send the request and get the response
        let response = request.send().await.unwrap();
        // get the response body in json format
        let body = response.text().await.unwrap();
        // deserialize the json body into the AccessTokenResponse struct
        let device_code_response: DeviceCodeResponse = serde_json::from_str(&body).unwrap();
        // let access_token = access_token_response.access_token;
        println!(
            "Follow the link to login: {}",
            device_code_response.verification_uri_complete
        );

        Ok(device_code_response.device_code)
    }
}

#[tokio::main]
async fn main() {
    let device_code = DeviceCodeResponse::get_device_code().await.unwrap();
    let mut sp = Spinner::new(Spinners::Dots, "Waiting for auth...".into());
    let access_token_response = AccessTokenResponse::get_access_token(device_code)
        .await
        .unwrap();
    let access_token: AccessTokenResponse = serde_json::from_str(&access_token_response).unwrap();
    println!("Access token response {:#?}", access_token.access_token);

    let client = reqwest::Client::new();
    let request = client.get("https://auth.rosnovsky.us/api/v2/users").header(
        "Authorization",
        format!("Bearer {}", access_token.access_token.unwrap()),
    );
    //send the request and get the response
    let response = request.send().await.unwrap();
    println!("Response status: {}", response.status());
    // get the response body in json format
    let body = response.text().await.unwrap();

    sp.stop();
    println!("{:#?}", body);
}
