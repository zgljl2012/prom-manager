use std::collections::HashMap;

use actix_web::{post, web::{self, Data}, error, HttpResponse, Error};
use log::{error as log_error, debug};
use serde::{Serialize, Deserialize};
use serde_json::json;
use futures::StreamExt;
use tokio::task;

use crate::state::AppState;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WechatText {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WechatRequest {
    msgtype: String,
    text: WechatText
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// Generate message
/// # Examples
/// 
/// ```ignore
/// #[derive(Debug, debug)
/// let notification = ...
/// generate_message(&notification)
/// ```
/// 
fn generate_message (n: &Notification) -> Option<String> {
    if n.alerts.len() == 0{
        return None
    }
    let mut s = String::new();
    let mut i = 0;
    let default_service = "Unknown service".to_string();
    let default_instance = "Unknown instance".to_string();
    let default_severity = "Unknown severity".to_string();
    let default_summary = "Empty summary".to_string();
    while i < n.alerts.len() {
        let alert = &n.alerts[i];
        let service = alert.labels.get("service").unwrap_or(&default_service);
        let instance = alert.labels.get("instance").unwrap_or(&default_instance);
        let severity = alert.labels.get("severity").unwrap_or(&default_severity);
        let summary = alert.annotations.get("summary").unwrap_or(&default_summary);
        let alert_msg= format!("Services: {:?}\nInstance: {:?}\nSeverity: {:?}\n----------\n{:?}\n", service, instance, severity, summary);
        if s.len() == 0 {
            s = alert_msg;
        } else {
            s = format!("{}\n{}", s, alert_msg);
        }
        i = i + 1;
    }
    Some(s)
}

async fn robot_handlers (state: &Data<AppState>, msg: String) {
    // Wechat robot handlers
    match &state.wechat_robot {
        Some(robot) => {
            debug!("Send {:?} to {:?}", msg, robot);
            let robot_url = robot.clone();
            let _ = task::spawn_blocking(move || {
                // async fn run(msg: String, robot: String) {
                //     let wechat_req = WechatRequest {
                //         msgtype: "text".to_string(),
                //         text: WechatText { content: msg }
                //     };
                //     let serialized = serde_json::to_string(&wechat_req).unwrap();
                //     let client = reqwest::Client::new();
                //     let r = client.clone().post(robot).header("Context-Type", "application/json")
                //         .body(serialized).send().await;
                //     match r {
                //         Ok(r) => {
                //             if r.status() != 200 {
                //                 log_error!("Send alert to wechat robot hook failed: {:?}", r.text().await.unwrap());
                //             }
                //         },
                //         Err(e) => log_error!("Send alert to wechat failed: {:?}", e)
                //     };
                // }
                // futures::join!(run(msg, robot_url.clone()));
            }).await;
        },
        None => {},
    }
}

#[post("/prometheus/hook")]
pub async fn prometheus_hook(state: Data<AppState>, mut payload: web::Payload) -> Result<HttpResponse, Error> {
    debug!("Prometheus hook called");
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
    let msg = generate_message(&obj);
    match msg {
        Some(msg) => robot_handlers(&state, msg).await,
        None => {},
    }
    // robot handlers
    Ok(HttpResponse::Ok().json(json!({ "result": true })))
}
