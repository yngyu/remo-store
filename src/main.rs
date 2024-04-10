use std::{env, error::Error};

use reqwest::{
    self,
    header::{HeaderMap, AUTHORIZATION},
};

const NATURE_API_ENDPOINT: &str = "https://api.nature.global/1/devices";
const PLACE: &str = "リビング";

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    env_logger::init();

    let token = env::var("NATURE_TOKEN").expect("Could not get environment variable NATURE_TOKEN.");
    let authorization = format!("Bearer {}", token);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, authorization.parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let body: Vec<serde_json::Value> = client.get(NATURE_API_ENDPOINT).send()?.json()?;

    let mut event = serde_json::Map::new();

    for device_info in body {
        let place = device_info["name"]
            .as_str()
            .expect("respond must have name field");
        if place != PLACE {
            continue;
        }

        event = device_info["newest_events"]["te"]
            .as_object()
            .expect("respond must have newest_events and te field")
            .clone();
        break;
    }

    let time = event["created_at"].as_str().expect("there must be created_at field");
    let temperature = event["val"].as_f64().expect("there must be val field");

    dbg!(&time, &temperature);

    Ok(())
}
