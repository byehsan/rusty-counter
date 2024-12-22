use serde::Deserialize;
use std::{env, io::Write, io::Read};
use log::{info, warn, error};

pub struct Counter {
    value: std::sync::Mutex<i32>,
    pub file_path: String,
    pub backup_path: String,
}

impl Counter {
    pub fn new(file_path: &str, backup_path: &str) -> Self {
        Self {
            value: std::sync::Mutex::new(0),
            file_path: file_path.to_string(),
            backup_path: backup_path.to_string(),
        }
    }

    pub fn load_or_initialize(&self) -> bool {
        let mut value = self.value.lock().unwrap();
    
        // Ensure the directory exists
        if let Some(parent_dir) = std::path::Path::new(&self.file_path).parent() {
            if let Err(err) = std::fs::create_dir_all(parent_dir) {
                warn!("Failed to create directory {}: {}", parent_dir.display(), err);
            }
        }
    
        // Ensure the main file exists
        if !std::path::Path::new(&self.file_path).exists() {
            if let Err(err) = std::fs::File::create(&self.file_path) {
                warn!("Failed to create main file {}: {}", self.file_path, err);
            } else {
                info!("Main file created: {}", self.file_path);
            }
        }
    
        // Try to open and read the main file
        if let Ok(mut file) = std::fs::File::open(&self.file_path) {
            let mut buffer = [0u8; 4];
            if file.read_exact(&mut buffer).is_ok() {
                *value = i32::from_be_bytes(buffer);
                info!("Loaded value from main file: {}", *value);
                return true;
            } else {
                warn!("Failed to read from main file, trying backup");
            }
        }
    
        // Ensure the backup file exists
        if !std::path::Path::new(&self.backup_path).exists() {
            if let Err(err) = std::fs::File::create(&self.backup_path) {
                warn!("Failed to create backup file {}: {}", self.backup_path, err);
            } else {
                info!("Backup file created: {}", self.backup_path);
            }
        }
    
        // Try to open and read the backup file
        if let Ok(mut backup_file) = std::fs::File::open(&self.backup_path) {
            let mut buffer = [0u8; 4];
            if backup_file.read_exact(&mut buffer).is_ok() {
                *value = i32::from_be_bytes(buffer);
                info!("Loaded value from backup file: {}", *value);
                self.save(); // Save to main file if loaded from backup
                return true;
            } else {
                warn!("Failed to read from backup file");
            }
        }
    
        // Initialize to default if no file exists
        *value = 0;
        info!("Initialized value to default (0)");
        self.save();
        false
    }
    
    

    pub fn increment(&self) {
        let mut value = self.value.lock().unwrap();
        *value += 1;
        info!("Incremented value to: {}", *value);
        self.save();
    }

    pub fn decrement(&self) {
        let mut value = self.value.lock().unwrap();
        *value -= 1;
        info!("Decremented value to: {}", *value);
        self.save();
    }

    pub fn get(&self) -> i32 {
        let value = *self.value.lock().unwrap();
        info!("Retrieved value: {}", value);
        value
    }

    pub fn backup(&self) -> bool {
        let value = *self.value.lock().unwrap();
        match std::fs::File::create(&self.backup_path) {
            Ok(mut file) => {
                if file.write_all(&value.to_be_bytes()).is_ok() {
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
    fn save(&self) {
        let value = *self.value.lock().unwrap();
        info!("Attempting to save value: {}", value);
    
        match std::fs::File::create(&self.file_path) {
            Ok(mut file) => {
                if file.write_all(&value.to_be_bytes()).is_ok() {
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
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid BACKUP_INTERVAL value"),
            service_ip: env::var("SERVICE_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
            service_port: env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("Invalid SERVICE_PORT value"),
        })
    }
}
