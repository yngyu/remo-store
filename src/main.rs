use std::{env, error::Error};

use chrono::DateTime;
use reqwest::{
    self,
    header::{HeaderMap, AUTHORIZATION},
};

const NATURE_API_ENDPOINT: &str = "https://api.nature.global/1/devices";
const PLACE_JA: &str = "リビング";
const PLACE_TAG: &str = "Living";
const SENSOR_TAG: &str = "Nature-Remo-mini-2";

fn main() -> Result<(), Box<(dyn Error + 'static)>> {
    env_logger::init();

    let nature_token =
        env::var("NATURE_TOKEN").expect("Could not get environment variable NATURE_TOKEN.");
    let influxdb_host =
        env::var("INFLUXDB_HOST").expect("Could not get environment variable INFLUXDB_HOST.");
    let influxdb_token =
        env::var("INFLUXDB_TOKEN").expect("Could not get environment variable INFLUXDB_TOKEN.");
    let influxdb_org =
        env::var("INFLUXDB_ORG").expect("Could not get environment variable INFLUXDB_ORG.");
    let influxdb_bucket =
        env::var("INFLUXDB_BUCKET").expect("Could not get environment variable INFLUXDB_BUCKET.");

    let authorization = format!("Bearer {}", nature_token);
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
        if place != PLACE_JA {
            continue;
        }

        event = device_info["newest_events"]["te"]
            .as_object()
            .expect("respond must have newest_events and te field")
            .clone();
        break;
    }

    let time = event["created_at"]
        .as_str()
        .expect("there must be created_at field");
    let unixtime = DateTime::parse_from_rfc3339(time)
        .expect("should be rfc3339 str")
        .timestamp();
    let temperature = event["val"].as_f64().expect("there must be val field");

    let influxdb_end_point = format!(
        "{}/api/v2/write?org={}&bucket={}",
        influxdb_host, influxdb_org, influxdb_bucket
    );
    let post_field = format!(
        "{},sensor_id={} temperature={} {}000000000",
        PLACE_TAG, SENSOR_TAG, temperature, unixtime
    );

    let authorization = format!("Token {}", influxdb_token);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, authorization.parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    client.post(influxdb_end_point).body(post_field).send()?;

    Ok(())
}
