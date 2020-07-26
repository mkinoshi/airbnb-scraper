use std::fs::File;
use std::io::prelude::*;
use tera::{Context, Tera};

pub struct Email;
use crate::RoomListing;
use crate::RoomListingItem;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("src/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

impl Email {
    fn format_email(results: &Vec<RoomListingItem>) -> Option<String> {
        let room_listing = RoomListing {
            listing: results.as_slice().to_vec(),
        };
        let room_listing_context =
            Context::from_serialize(&room_listing).expect("rendering context");
        let rendered = TEMPLATES.render("product.html", &room_listing_context);

        if !rendered.is_ok() {
            return None;
        }

        return Some(rendered.unwrap());
    }

    pub async fn send_email(
        email: &String,
        results: &Vec<RoomListingItem>,
    ) -> Result<(), reqwest::Error> {
        let api_key = "<Place your api key>";
        let mailgun_endpoint = format!("https://api:{}@api.mailgun.net/v3", api_key);
        let domain = "email.airnotify.info";
        let content = Self::format_email(results);
        if let Some(c) = content {
            let format = [
                ("from", "email@airnotify.info"),
                ("to", email),
                ("subject", "New Listings"),
                ("html", &c),
            ];
            let client = reqwest::Client::new();
            client
                .post(&format!("{}/{}/messages", mailgun_endpoint, domain).to_string())
                .form(&format)
                .send()
                .await?;
        }
        Ok(())
    }
}

#[test]
fn test_format_email() {
    let mut data = String::new();
    File::open("src/test/data.json")
        .unwrap()
        .read_to_string(&mut data)
        .expect("It should work");
    let room_listing: Vec<RoomListingItem> = serde_json::from_str(&data).unwrap();

    let res = Email::format_email(&room_listing);
    let file = File::create("src/templates/test.html");
    file.unwrap()
        .write_all(&res.unwrap().as_bytes())
        .expect("Successfully Written");
}

//#[tokio::test]
//async fn send_email_test() {
//let handle = Email::send_email().await.expect("my function");
