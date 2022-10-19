use std::collections::HashMap;

use actix_web::{post, web, error, HttpResponse, Error};
use log::info;
use serde::{Serialize, Deserialize};
use serde_json::json;
use futures::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Alert {
    status: String,
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    startsAt: String,
    endsAt: String,
    generatorURL: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Notification {
    receiver: String,
    status: String,
    alerts: Vec<Alert>,
    groupLabels: HashMap<String, String>,
    commonLabels: HashMap<String, String>,
    commonAnnotations: HashMap<String, String>,
    externalURL: String,
    version: String
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/prometheus/hook")]
pub async fn prometheus_hook(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    info!("Prometheus hook called");
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Notification>(&body)?;
    info!("{:?}", obj);
    Ok(HttpResponse::Ok().json(json!({ "result": true })))
}
