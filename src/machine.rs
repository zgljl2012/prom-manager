use actix_web::{
    delete, get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::state::{AppState, Machine};

#[post("/machine")]
pub async fn add_machine(state: Data<AppState>, payload: web::Payload) -> impl Responder {
    match Machine::from_payload(payload).await {
        Ok(machine) => {
            state
                .machine_manager
                .lock()
                .unwrap()
                .add_machine(machine.clone());
            HttpResponse::Ok().json(json!({"result": true}))
        }
        Err(err) => HttpResponse::BadRequest()
            .json(json!({ "error": format!("Add machine failed: {:?}", err) })),
    }
}

#[get("/machines")]
pub async fn list_machines(state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(state.machine_manager.lock().unwrap().to_vec())
}

#[delete("/machine/{target}")]
pub async fn remove_machine(state: Data<AppState>, target: web::Path<usize>) -> impl Responder {
    state.machine_manager.lock().unwrap().remove_machine(target.to_be());
    HttpResponse::Ok().json(json!({"result": true}))
}
