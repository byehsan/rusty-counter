use rocket::State;
use std::sync::Arc;

use crate::storage::Counter;

#[get("/get")]
pub fn get_value(counter: &State<Arc<Counter>>) -> String {
    format!("{}", counter.get())
}

#[post("/increment")]
pub fn increment(counter: &State<Arc<Counter>>) -> String {
    counter.increment();
    format!("Incremented counter to {}", counter.get())
}

#[post("/decrement")]
pub fn decrement(counter: &State<Arc<Counter>>) -> String {
    counter.decrement();
    format!("Incremented counter to {}", counter.get())
}
