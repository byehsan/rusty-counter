use serde::Deserialize;
use std::fs::{File, create_dir_all};
use std::{env, io::Write, io::Read, io::BufReader};
use log::{info, warn, error};
use std::sync::Mutex;
pub struct Counter {
    value: std::sync::Mutex<String>,
    pub file_path: String,
    pub backup_path: String,
}

impl Counter {
    // Constructor
    pub fn new(file_path: &str, backup_path: &str) -> Self {
        Self {
            value: Mutex::new("0".to_string()), // Default value as a string
            file_path: file_path.to_string(),
            backup_path: backup_path.to_string(),
        }
    }

    // Load or initialize the file content
    pub fn load_or_initialize(&self) -> bool {
        let mut value = self.value.lock().unwrap();
    
        // Ensure the directory exists
        if let Some(parent_dir) = std::path::Path::new(&self.file_path).parent() {
            if let Err(err) = create_dir_all(parent_dir) {
                warn!("Failed to create directory {}: {}", parent_dir.display(), err);
            }
        }
    
        // Ensure the main file exists and initialize with "0" if not
        if !std::path::Path::new(&self.file_path).exists() {
            // Create and initialize the file with "0"
            if let Err(err) = File::create(&self.file_path) {
                warn!("Failed to create and initialize main file {}: {}", self.file_path, err);
            } else {
                info!("Main file created: {}", self.file_path);
            }
        }
    
        // Try to open and read the main file as a string
        if let Ok(file) = File::open(&self.file_path) {
            let mut reader = BufReader::new(file);
            let mut content = String::new();
            if reader.read_to_string(&mut content).is_ok() {
                // Try parsing the content as an i32
                if let Ok(parsed_value) = content.trim().parse::<i32>() {
                    *value = parsed_value.to_string();
                    info!("Loaded value from main file: {}", *value);
                    return true;
                } else {
                    warn!("Failed to parse value as i32 from main file, trying backup");
                }
            } else {
                warn!("Failed to read from main file, trying backup");
            }
        }
    
        // Ensure the backup file exists and initialize with "0" if not
        if !std::path::Path::new(&self.backup_path).exists() {
            // Create and initialize the backup file with "0"
            if let Err(err) = File::create(&self.backup_path).and_then(|mut f| f.write_all(b"0")) {
                warn!("Failed to create and initialize backup file {}: {}", self.backup_path, err);
            } else {
                info!("Backup file created and initialized with '0': {}", self.backup_path);
            }
        }
    
        // Try to open and read the backup file as a string
        if let Ok(backup_file) = File::open(&self.backup_path) {
            let mut reader = BufReader::new(backup_file);
            let mut content = String::new();
            if reader.read_to_string(&mut content).is_ok() {
                *value = content.trim().to_string();
                info!("Loaded value from backup file: {}", *value);
                drop(value);
                self.save(); // Save to main file if loaded from backup
                return true;
            } else {
                warn!("Failed to read from backup file");
            }
        }
    
        // Initialize to default if no file exists or if reading fails
        *value = "0".to_string();
        info!("Initialized value to default ('0')");
        drop(value);
        self.save();
        false
    }

    // Increment the counter
    pub fn increment(&self) {
        let mut value = self.value.lock().unwrap();
        *value = (value.parse::<i32>().unwrap_or(0) + 1).to_string(); // Increment the value
        info!("Incremented value to: {}", *value);
        drop(value);
        self.save();
    }

    // Decrement the counter
    pub fn decrement(&self) {
        let mut value = self.value.lock().unwrap();
        *value = (value.parse::<i32>().unwrap_or(0) - 1).to_string(); // Decrement the value
        info!("Decremented value to: {}", *value);
        drop(value);
        self.save();
    }

    // Get the current value
    pub fn get(&self) -> String {
        let value = self.value.lock().unwrap().clone();
        info!("Retrieved value: {}", value);
        value
    }

    // Backup the current value to the backup file
    pub fn backup(&self) -> bool {
        let value = self.value.lock().unwrap().clone();
        match File::create(&self.backup_path) {
            Ok(mut file) => {
                if file.write_all(value.as_bytes()).is_ok() {
                    info!("Backup saved to {}", self.backup_path);
                    true
                } else {
                    error!("Failed to write backup to {}", self.backup_path);
                    false
                }
            }
            Err(err) => {
                error!("Failed to create backup file: {}", err);
                false
            }
        }
    }

    // Save the current value to the main file
    fn save(&self) {
        let value = self.value.lock().unwrap().clone();
        info!("Attempting to save value: {}", value);
    
        match File::create(&self.file_path) {
            Ok(mut file) => {
                if file.write_all(value.as_bytes()).is_ok() {
                    info!("Value saved to {}", self.file_path);
                } else {
                    error!("Failed to write to main file: {}", self.file_path);
                }
            }
            Err(err) => {
                error!("Failed to create main file: {}. Error: {}", self.file_path, err);
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_file_path: String,
    pub backup_file_path: String,
    pub backup_interval: u64,
    pub service_ip: String,
    pub service_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            data_file_path: env::var("DATA_FILE_PATH").unwrap_or_else(|_| "./local_dir/data.bin".to_string()),
            backup_file_path: env::var("BACKUP_FILE_PATH").unwrap_or_else(|_| "./local_dir/backup.bin".to_string()),
            backup_interval: env::var("BACKUP_INTERVAL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .expect("Invalid BACKUP_INTERVAL value"),
            service_ip: env::var("SERVICE_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            service_port: env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("Invalid SERVICE_PORT value"),
        })
    }
}
