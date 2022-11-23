mod api_client;
mod configuration_service;

use std::error::Error;
use std::{env, fs, thread};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::StatusCode;

use api_client::TaskStatus;

pub struct Config {
    pub output_dir: String,
    pub space_id: String,
    pub email: String,
    pub password: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let output_dir = match args.next() {
            Some(arg) => arg,
            None => return Err("Output directory argument missing"),
        };

        let space_id = match args.next() {
            Some(arg) => arg,
            None => return Err("Space ID argument missing"),
        };

        let email = match env::var("NOTION_EMAIL") {
            Ok(value) => value,
            Err(_) => return Err("NOTION_EMAIL env missing"),
        };

        let password = match env::var("NOTION_PASSWORD") {
            Ok(value) => value,
            Err(_) => return Err("NOTION_PASSWORD env missing"),
        };

        Ok(Config { output_dir, space_id, email, password })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>>  {
    let mut config_service = configuration_service::ConfigurationService::new();
    let api_client = api_client::ApiClient::new();

    println!("Creating backup for space {} in {}", config.space_id, config.output_dir);

    let mut token = match config_service.get_token() {
        Some(token) => {
            println!("Using existing auth token");
            token
        },
        None => {
            println!("Getting new auth token");
            let token = api_client.login(&config.email, &config.password)?;
            config_service.set_token(&token)
        }
    };

    println!("Validating token");
    match api_client.validate_token(token) {
        Ok(_) => {
            println!("Auth token is valid");
        },
        Err(e) => {
            match e.status() {
                Some(status_code) => {
                    if status_code == StatusCode::UNAUTHORIZED {
                        println!("Auth token is invalid, getting new one");
                        let local_token = api_client.login(&config.email, &config.password)?;
                        token = config_service.set_token(&local_token);
                    } else {
                        Err(e)?;
                    }
                }
                None => Err(e)?
            }
        }
    }

    println!("Requesting space export");
    let task_id = api_client.export_space(&config.space_id, token)?;

    let mut task_status: TaskStatus;

    loop {
        task_status = api_client.get_task_status(&task_id, token)?;

        if task_status.value == "complete" {
            println!("Space export task is complete");
            break;
        }

        println!("Space export task is in progress");

        thread::sleep(Duration::from_secs(5));
    }

    println!("Downloading backup file");

    let current_time_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let filename = format!("NotionBackup-{}.zip", current_time_millis);

    let output_path = fs::canonicalize(&config.output_dir)?;
    let output_file = output_path.as_path().join(filename);

    let resp = reqwest::blocking::get(task_status.export_url)?.bytes()?;
    fs::write(&output_file, &resp)?;

    println!("Backup stored at {}", output_file.display());

    Ok(())
}
