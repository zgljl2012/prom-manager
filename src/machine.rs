use actix_web::{Responder, get, post, delete, patch};


#[post("/machine")]
pub async fn add_machine() -> impl Responder {
    format!("add")
}

#[get("/machines")]
pub async fn list_machines() -> impl Responder {
    format!("add")
}

#[delete("/remove")]
pub async fn remove_machine() -> impl Responder {
    format!("add")
}

#[patch("/update")]
pub async fn update_machine() -> impl Responder {
    format!("add")
}
