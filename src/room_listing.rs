use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomListingContent {
    pub host_type: Option<String>,
    pub title: Option<String>,
    pub rating: Option<String>,
    pub review_number: Option<String>,
    pub room_strucure: Option<String>,
    pub amenities: Option<String>,
    pub price: Option<String>,
}

impl RoomListingContent {
    pub fn update_room_listing(listing: &mut RoomListingContent, key: &str, value: &str) {
        match key {
            "host_type" => {
                listing.host_type = Some(value.to_string());
            }
            "title" => {
                listing.title = Some(value.to_string());
            }
            "rating" => {
                listing.rating = Some(value.to_string());
            }
            "review_number" => {
                listing.review_number = Some(value.to_string());
            }
            "room_strucure" => {
                listing.room_strucure = Some(value.to_string());
            }
            "amenities" => {
                listing.amenities = Some(value.to_string());
            }
            "price" => {
                listing.price = Some(value.to_string());
            }
            _ => (),
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomListingItem {
    pub url: String,
    pub image_url: String,
    pub content: RoomListingContent,
}

impl RoomListingItem {
    pub fn new() -> RoomListingItem {
        RoomListingItem {
            url: "".to_string(),
            image_url: "".to_string(),
            content: RoomListingContent {
                ..Default::default()
            },
        }
    }
}

fn extract_room_id(url: &String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/rooms/.*\?").unwrap();
    }

    let room_id = RE.captures(&url);
    let room_id = match room_id {
        Some(id) => {
            let room_id = id.get(0).map_or("", |m| m.as_str());
            let room_id = room_id.replace("/rooms/", "");
            room_id.replace("?", "")
        }
        _ => "".to_string(),
    };
    return room_id;
}

pub fn extract_new_listing(
    new_listing: Vec<RoomListingItem>,
    existing_listing: &Vec<RoomListingItem>,
) -> Vec<RoomListingItem> {
    new_listing
        .into_iter()
        .filter(|item| {
            let mut existing_listing_iter = existing_listing.into_iter();
            !existing_listing_iter.any(|existing_item| {
                extract_room_id(&existing_item.url) == extract_room_id(&item.url)
                    && existing_item.content.price == item.content.price
            })
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomListing {
    pub listing: Vec<RoomListingItem>,
}

impl RoomListing {
    pub fn new() -> RoomListing {
        RoomListing { listing: vec![] }
    }

    pub fn push(&mut self, item: RoomListingItem) {
        self.listing.push(item);
    }
}

#[test]
fn test_extract_new_listing_when_source_is_empty() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("file is opened");
    let room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let src_room_listing = vec![];
    let res = extract_new_listing(src_room_listing, &room_listing);
    assert_eq!(res.len(), 0)
}

#[test]
fn test_extract_new_listing_when_existing_listing_is_empty() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("file is opened");
    let src_room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let room_listing = vec![];
    let res = extract_new_listing(src_room_listing, &room_listing);
    assert_eq!(res.len(), 20)
}

#[test]
fn test_extract_new_listing_when_there_is_no_diff() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("file is opened");
    let src_room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let res = extract_new_listing(src_room_listing, &room_listing);
    assert_eq!(res.len(), 0)
}

#[test]
fn test_extract_new_listing_when_there_is_one_difference_in_url() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("file is opened");
    let mut src_room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    src_room_listing[0].url = "123".to_string();
    let res = extract_new_listing(src_room_listing, &room_listing);
    assert_eq!(res.len(), 1)
}

#[test]
fn test_extract_new_listing_when_there_is_one_difference_in_price() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("file is opened");
    let mut src_room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    let room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();
    src_room_listing[0].content.price = Some("123".to_string());
    let res = extract_new_listing(src_room_listing, &room_listing);
    assert_eq!(res.len(), 1)
}
