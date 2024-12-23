use rocket::{get, post, serde::json::Json, State};
use std::sync::Arc;

use crate::storage::Counter;

#[derive(serde::Serialize)]
pub struct Response {
    value: i32,
}

#[get("/get")]
pub fn get_value(counter: &State<Arc<Counter>>) -> Json<Response> {
    Json(Response {
        value: counter.get(),
    })
}

#[post("/increment")]
pub fn increment(counter: &State<Arc<Counter>>) -> Json<Response> {
    counter.increment();
    Json(Response {
        value: counter.get(),
    })
}

#[post("/decrement")]
pub fn decrement(counter: &State<Arc<Counter>>) -> Json<Response> {
    counter.decrement();
    Json(Response {
        value: counter.get(),
    })
}
