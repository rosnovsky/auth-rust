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
    access_token: String,
}

impl AccessTokenResponse {
    async fn get_access_token(device_code: String) -> Result<String, reqwest::Error> {
        async fn poll_fn(device_code: &String) -> Result<AccessTokenResponse, reqwest::Error> {
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
            let access_code_response: AccessTokenResponse = serde_json::from_str(&body).unwrap();
            Ok(access_code_response)
        }
        let mut access_token_response = poll_fn(&device_code).await;
        while access_token_response.as_ref().unwrap() {
            access_token_response = poll_fn(&device_code).await;
            thread::sleep(std::time::Duration::from_millis(500));
        }
        Ok(access_token_response.unwrap().access_token)

        // let access_token_response = poll_fn(&device_code).await;
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
        .body("client_id=59WtMZZWhn2sXvNluQUB45qXAg7NHsYt&scope=openid%20email%20profile%20offline_access&audience=/blog/");

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
    println!("{:#?}", access_token_response);
    // let access_token_object: String = serde_json::from_str(&access_token_response).unwrap();
    // println!("{}", access_token_object);
    // let client = reqwest::Client::new();
    // let request = client
    //     .get("https://auth.rosnovsky.us/api/v2/users")
    //     .header("Authorization", format!("Bearer {}", access_token.));
    // //send the request and get the response
    // let response = request.send().await.unwrap();
    // // get the response body in json format
    // let body = response.text().await.unwrap();
    // // deserialize the json body into the Value struct
    // let users: Vec<User> = serde_json::from_str(&body).unwrap();
    sp.stop();

    // Print the response
    println!("{:#?}", "body");
}
