use crate::room_listing::RoomListingItem;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::str;
use tempfile::tempdir;
use uuid::Uuid;
pub struct Storage;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchConfig {
    pub id: Uuid,
    pub url: String,
    pub result: Option<Vec<RoomListingItem>>,
    pub email: String,
}

impl Storage {
    pub fn load_all_search_files() -> Vec<String> {
        let objects_list = Command::new("gsutil")
            .args(&["ls", "gs://airnotify-dev/*"])
            .output();
        match objects_list {
            Ok(v) => Self::process_output(v.stdout),
            Err(e) => {
                println!("Error at gsutil ls: {}", e);
                vec!["".to_string()]
            }
        }
    }

    fn process_output(result: Vec<u8>) -> Vec<String> {
        let result_string = str::from_utf8(&result).unwrap().to_string();
        result_string
            .split("\n")
            .map(|s| s.to_string())
            .filter(|v| v != "")
            .collect()
    }

    pub fn load_search_config(search_file: &String) -> Result<SearchConfig, Box<dyn Error>> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(format!("{}.json", "test"));
        Command::new("gsutil")
            .args(&["cp", "-r", search_file, &file_path.to_str().unwrap()])
            .output()?;

        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let u = serde_json::from_reader(reader)?;
        Ok(u)
    }

    pub fn add_new_config(url: String, email: String) -> Result<(), Box<dyn Error>> {
        let id = Uuid::new_v4();
        let new_config = SearchConfig {
            id,
            url,
            email,
            result: None,
        };
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(format!("{}.json", id));
        let file = File::create(&file_path)?;
        serde_json::to_writer(&file, &new_config)?;
        Command::new("gsutil")
            .args(&[
                "cp",
                "-r",
                &file_path.to_str().unwrap(),
                "gs://airnotify-dev",
            ])
            .output()?;
        Ok(())
    }
}
