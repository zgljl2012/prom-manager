use actix_web::{
    delete, get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::state::{AppState, Machine};

#[post("/service")]
pub async fn add_service(state: Data<AppState>, payload: web::Payload) -> impl Responder {
    match Machine::from_payload(payload).await {
        Ok(service) => {
            state
                .service_manager
                .lock()
                .unwrap()
                .add_machine(service.clone());
            HttpResponse::Ok().json(json!({"result": true}))
        }
        Err(err) => HttpResponse::BadRequest()
            .json(json!({ "error": format!("Add service failed: {:?}", err) })),
    }
}

#[get("/services")]
pub async fn list_services(state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(state.service_manager.lock().unwrap().to_vec())
}

#[delete("/service/{target}")]
pub async fn remove_service(state: Data<AppState>, target: web::Path<String>) -> impl Responder {
    state.service_manager.lock().unwrap().remove_machine(&target.to_string());
    HttpResponse::Ok().json(json!({"result": true}))
}
