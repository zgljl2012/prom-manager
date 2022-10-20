use std::collections::HashMap;

use log::{error as log_error, debug, info};
use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;

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

fn robot_handlers (rt: &Runtime, msg: String) {
    // Wechat robot handlers
    let robot = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=d0b8ea9c-bebc-4e71-a85e-d1cd3b1f101e".to_string();
    debug!("Send {:?} to {:?}", msg, robot);
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(async {
    //     info!("Received {:?}", msg);
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
    // });
}

pub fn prometheus_hook(request: http::Request) -> http::Response {
    debug!("Prometheus hook called");
    // body is loaded, now we can deserialize serde-json
    // let obj = serde_json::from_str::<Notification>(&request.body.unwrap()).unwrap();
    // let msg = generate_message(&obj);
    // match msg {
    //     Some(msg) => robot_handlers(&request.rt, msg),
    //     None => {},
    // }
    // robot handlers
    http::Response { status: 200, body: "{\"result\": \"ok\"}".to_string()}
}
