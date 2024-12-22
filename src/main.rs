#[macro_use]
extern crate rocket;

use std::sync::Arc;
use std::time::Duration;

mod routes;
mod storage;

use storage::{Config, Counter};
use log::{info, warn, error};

#[launch]
fn rocket() -> _ {
    // Initialize the logger
    env_logger::init();

    // Load configuration from environment variables into a Config struct
    let config = Config::from_env().expect("Failed to load configuration");

    info!("Configuration loaded: {:?}", config);

    // Initialize Counter
    let counter = Arc::new(Counter::new(&config.data_file_path, &config.backup_file_path));
    if counter.load_or_initialize() {
        info!("Counter initialized successfully");
    } else {
        warn!("Counter initialization failed, using default value (0)");
    }

    // Start backup thread
    let counter_clone = Arc::clone(&counter);
    let backup_duration = Duration::from_secs(config.backup_interval);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(backup_duration);
            if counter_clone.backup() {
                info!("Backup successful");
            } else {
                error!("Backup failed");
            }
        }
    });

    // Start Rocket server
    info!(
        "Starting server on {}:{}",
        config.service_ip, config.service_port
    );
    rocket::custom(
        rocket::Config::figment()
            .merge(("address", config.service_ip))
            .merge(("port", config.service_port)),
    )
    .manage(counter)
    .mount("/", routes![routes::get_value, routes::increment, routes::decrement])
}
