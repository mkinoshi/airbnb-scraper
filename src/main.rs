//use diesel::pg::PgConnection;
//use diesel::prelude::*;
//use scraper::models::{SeachForm, Search};
//use scraper::db::Db;
use scraper::email::Email;
use scraper::room_listing::extract_new_listing;
use scraper::storage::Storage;
use scraper::Scraper;
use std::error::Error;
use std::fs::File;
use std::process::Command;
use tempfile::tempdir;

async fn execute_sequence(search_file: &String) -> Result<(), Box<dyn Error>> {
    let mut search_config = Storage::load_search_config(search_file)?;
    let url = &search_config.url;
    let email = &search_config.email;
    let res = Scraper::scrape(&url).await?;
    let search_result = search_config.result.clone();
    let new_listings = match search_config.result {
        Some(r) => extract_new_listing(res, &r),
        _ => res,
    };
    println!("THE NEW LISTING: {:?}", new_listings);
    if new_listings.len() > 0 {
        Email::send_email(&email, &new_listings).await?;
        let new_result = match search_result {
            Some(r) => [&new_listings[..], &r[..]].concat(),
            _ => new_listings,
        };
        search_config.result = Some(new_result);
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(format!("{}.json", search_config.id));
        let write_res = serde_json::to_writer(&File::create(&file_path).unwrap(), &search_config);
        if let Ok(_) = write_res {
            Command::new("gsutil")
                .args(&[
                    "cp",
                    "-r",
                    &file_path.to_str().unwrap(),
                    "gs://airnotify-dev",
                ])
                .output()?;
        }
    }
    println!("Finished!");
    Ok(())
}

#[tokio::main]
async fn main() {
    //let connection = Db::establish_connection();
    //let s = Db::load_all_searches(&connection);
    let s = Storage::load_all_search_files();
    for search in s.iter() {
        let res = execute_sequence(&search).await;
        if let Err(_) = res {
            println!("Error occured for Search<{}>", search);
        }
    }
    //Storage::add_new_config("https://www.airbnb.com/s/Buenos-Aires--Autonomous-City-of-Buenos-Aires--Argentina/homes?tab_id=home_tab&refinement_paths%5B%5D=%2Fhomes&source=structured_search_input_header&search_type=filter_change&ne_lat=-34.569339713683085&ne_lng=-58.42145347625734&sw_lat=-34.59520326375472&sw_lng=-58.453039169616716&zoom=15&search_by_map=true&place_id=ChIJvQz5TjvKvJURh47oiC6Bs6A&query=Buenos%20Aires%2C%20Autonomous%20City%20of%20Buenos%20Aires&checkin=2020-05-17&checkout=2020-06-17&adults=2&room_types%5B%5D=Entire%20home%2Fapt&min_beds=0&property_type_id%5B%5D=2&property_type_id%5B%5D=1&min_bedrooms=2".to_string(), "mkinoshi12@gmail.com".to_string());
}
