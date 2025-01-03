#[macro_use]
extern crate rocket;

use dotenv::dotenv;
use std::sync::Arc;
use std::time::Duration;

mod routes;
mod storage;

use log::{error, info, warn};
use storage::{Config, Counter};

#[launch]
fn rocket() -> _ {
    // Load environment variables from .env file
    match dotenv() {
        Ok(_) => println!("Successfully loaded .env file"),
        Err(_) => println!("Failed to load .env file"),
    }

    // Initialize the logger
    env_logger::init();


    // Print help for .env variables
    print_env_help();

    // Load configuration from environment variables into a Config struct
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            std::process::exit(1); // Exit the application if config loading fails
        }
    };

    info!("Configuration loaded: {:?}", config);

    // Initialize Counter
    let counter = Arc::new(Counter::new(
        &config.data_file_path,
        &config.backup_file_path,
    ));
    if counter.load_or_initialize() {
        info!("Counter initialized successfully");
    } else {
        warn!("Counter initialization failed, using default value (0)");
    }

    // Start backup thread
    let counter_clone = Arc::clone(&counter);
    let backup_duration = Duration::from_secs(config.backup_interval);
    std::thread::spawn(move || loop {
        std::thread::sleep(backup_duration);
        if counter_clone.backup() {
            info!("Backup successful");
        } else {
            error!("Backup failed");
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
    .mount(
        "/",
        routes![routes::get_value, routes::increment, routes::decrement],
    )
}

/// Prints the help for the environment variables used in the application
fn print_env_help() {
    println!("\nEnvironment Variables:");
    println!("DATA_FILE_PATH: The path to the data file (default: './local_dir/data.bin')");
    println!("BACKUP_FILE_PATH: The path to the backup file (default: './local_dir/backup.bin')");
    println!("BACKUP_INTERVAL: The interval for backup in seconds (default: 5)");
    println!("SERVICE_IP: The IP address of the service (default: '127.0.0.1')");
    println!("SERVICE_PORT: The port of the service (default: '8000')");
}
