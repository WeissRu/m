use inquire::{Select, Confirm};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process;
use crossterm::style::Stylize;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    source_dir: Vec<String>,
    time_limit: u64,
}

#[derive(Debug, Clone)]
struct FileInfo {
    path: PathBuf,
    name: String,
    size: u64,
    created_time: String,
    created_timestamp: u64,
}

fn main() {
    // Read configuration
    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read configuration: {}", e);
            process::exit(1);
        }
    };

    // Find recently created files
    let files = match find_recent_files(&config) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Failed to find files: {}", e);
            process::exit(1);
        }
    };

    if files.is_empty() {
        println!("{}", format!("No new files found in the last {} minutes", config.time_limit).red());
        return;
    }

    // Present files for selection
    let selected_file = match select_file(files) {
        Ok(file) => file,
        Err(_) => {
            println!("No file selected");
            return;
        }
    };

    // Move the selected file
    if let Err(e) = move_file(&selected_file) {
        eprintln!("Failed to move file: {}", e);
        process::exit(1);
    }
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".config")
        .join("m")
        .join("m.json");

    if !config_path.exists() {
        // Create default config if it doesn't exist
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        
        let home_dir = dirs::home_dir().unwrap();
        let default_config = Config {
            source_dir: vec![
                format!("{}/Downloads/", home_dir.display()),
                format!("{}/Sync/", home_dir.display()),
            ],
            time_limit: 20,
        };
        
        let json_content = serde_json::to_string_pretty(&default_config)?;
        
        fs::write(&config_path, json_content)?;
        println!("Created default configuration file at: {}", config_path.display());
        return Ok(default_config);
    }

    let json_content = fs::read_to_string(&config_path)?;
    let config: Config = serde_json::from_str(&json_content)?;
    
    Ok(config)
}

fn find_recent_files(config: &Config) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let time_limit_seconds = config.time_limit * 60;
    
    // Iterate over source directories
    for source_dir in &config.source_dir {
        let source_path = Path::new(source_dir);
        if !source_path.exists() {
            continue;
        }
        
        let entries = fs::read_dir(source_path)?;
        
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            // Skip hidden files and directories
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            
            if !metadata.is_file() {
                continue;
            }
            
            // Check if file was created within the time limit
            let created_time = metadata.created()?
                .duration_since(UNIX_EPOCH)?
                .as_secs();
            
            if current_time - created_time <= time_limit_seconds {
                let file_path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                let size = metadata.len();
                
                // Format creation time as HH:MM
                let created_datetime = chrono::DateTime::from_timestamp(created_time as i64, 0)
                    .unwrap_or_default();
                let time_str = created_datetime.format("%H:%M").to_string();
                
                files.push(FileInfo {
                    path: file_path,
                    name: file_name,
                    size,
                    created_time: time_str,
                    created_timestamp: created_time,
                });
            }
        }
    }
    
    // Sort by creation time (newest first)
    // files.sort_by(|a, b| b.path.cmp(&a.path));
    files.sort_by(|a, b| b.created_timestamp.cmp(&a.created_timestamp));
    
    Ok(files)
}

fn select_file(files: Vec<FileInfo>) -> Result<FileInfo, Box<dyn std::error::Error>> {
    let options: Vec<String> = files.iter().map(|file| {
        format!("{}\t{}\t{}", 
            file.created_time,
            format_size(file.size),
            file.name
        )
    }).collect();
    
    let selected = Select::new("Select a file to move:", options)
        .with_help_message("Use arrow keys to navigate, press Enter to select")
        .prompt()?;
    
    // Find the corresponding file info
    let selected_index = files.iter().position(|file| {
        format!("{}\t{}\t{}", 
            file.created_time,
            format_size(file.size),
            file.name
        ) == selected
    }).unwrap();
    
    Ok(files[selected_index].clone())
}

fn move_file(file_info: &FileInfo) -> Result<(), Box<dyn std::error::Error>> {
    let target_path = Path::new(&file_info.name);
    
    // Check if file already exists in current directory
    if target_path.exists() {
        let overwrite = Confirm::new(&format!("File '{}' already exists. Overwrite?", file_info.name))
            .with_default(false)
            .with_help_message("This will permanently replace the existing file")
            .prompt()?;
        
        if !overwrite {
            println!("Operation canceled");
            return Ok(());
        }
    }
    
    // Move the file
    fs::rename(&file_info.path, target_path)?;
    println!("{}", format!("Successfully moved '{}' to current directory", file_info.name).green());
    
    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{}B", bytes)
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}
