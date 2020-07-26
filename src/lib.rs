#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use fantoccini::{Client, Element, Locator};
use regex::Regex;
use serde_json::json;
use std::error::Error;
use webdriver::command::WebDriverCommand;

pub mod db;
pub mod email;
pub mod models;
pub mod room_listing;
pub mod schema;
pub mod storage;

use crate::room_listing::{RoomListing, RoomListingContent, RoomListingItem};

pub struct Scraper;
struct XpathConfig {
    xpath: Vec<String>,
    item_type: String,
}

impl XpathConfig {
    fn initialize() -> Vec<XpathConfig> {
        let room_listing_type_xpath = vec!["./div[2]/div[1]/div/div[1]/div".to_string()];
        let rating_xpath = vec!["./div[2]/div[1]/span/span/span[2]".to_string()];
        let review_number_xpath = vec!["./div[2]/div[1]/span/span/span[3]".to_string()];
        let title_xpath = vec!["./div[2]/div[1]/div/div[2]".to_string()];
        let room_strucure_xpath = vec!["./div[2]/div[3]".to_string()];
        let amenities_xpath = vec!["./div[2]/div[4]".to_string()];
        let price_xpath = vec![
            "./div[2]/div[5]/div[last()]/div/div/span/span".to_string(),
            "./div[2]/div[5]/div[last()]/div/span/span".to_string(),
        ];
        let mut content_xpaths: Vec<XpathConfig> = vec![];
        content_xpaths.push(XpathConfig {
            xpath: room_listing_type_xpath,
            item_type: "host_type".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: title_xpath,
            item_type: "title".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: rating_xpath,
            item_type: "rating".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: review_number_xpath,
            item_type: "review_number".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: room_strucure_xpath,
            item_type: "room_strucure".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: amenities_xpath,
            item_type: "amenities".to_string(),
        });
        content_xpaths.push(XpathConfig {
            xpath: price_xpath,
            item_type: "price".to_string(),
        });
        content_xpaths
    }
}

impl Scraper {
    async fn setup() -> Result<Client, fantoccini::error::NewSessionError> {
        let capabilities = json!({
            "capabilities": {
                "moz:firefoxOptions": {
                    "args": ["-headless"]
                }
            }
        });
        let client = Client::with_capabilities(
            "http://0.0.0.0:4444",
            capabilities["capabilities"].as_object().unwrap().clone(),
        )
        .await?;
        return Ok(client);
    }

    async fn select_url(
        element: &mut fantoccini::Element,
        path: &str,
    ) -> Result<Option<String>, fantoccini::error::CmdError> {
        let mut link_element = element.find(Locator::XPath(path)).await?;
        return Ok(link_element.attr("href").await?);
    }

    async fn select_img(
        element: &mut fantoccini::Element,
        first_path: &str,
        second_path: &str,
    ) -> Result<Option<String>, fantoccini::error::CmdError> {
        let mut img_element = element.find(Locator::XPath(first_path)).await;
        if let Ok(mut img) = img_element {
            return Ok(img.attr("style").await?);
        };
        let mut img_element = element.find(Locator::XPath(second_path)).await;
        if let Ok(mut img) = img_element {
            return Ok(img.attr("srcset").await?);
        };
        return Ok(None);
    }

    fn extract_url(room: &mut RoomListingItem, url_href: Option<String>) {
        if let Some(href) = url_href {
            room.url = href.clone();
        }
    }

    fn extract_img_url(room: &mut RoomListingItem, img_style: Option<String>) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"https.*jpg").unwrap();
        }

        if let Some(style) = img_style {
            let img_url = RE.captures(&style).unwrap();
            room.image_url = img_url[0].to_string();
        }
    }

    async fn scrape_text(element: &mut Element, result: &mut String, xpath: &str) {
        let inner_element_find = element.find(Locator::XPath(xpath)).await;
        if let Ok(mut v) = inner_element_find {
            let div_element = v.text().await;
            if let Ok(text) = div_element {
                result.push_str(&text);
            }
        }
    }

    pub async fn execute<'a>(
        c: &'a mut Client,
        url_path: &str,
        result: &mut Vec<RoomListingItem>,
    ) -> Result<&'a mut Client, fantoccini::error::CmdError> {
        c.goto(url_path).await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), url_path);

        let xpath_config = XpathConfig::initialize();
        let listing_container_xpath =
            "//a[contains(@href,'/rooms/')][contains(@data-check-info-section,'true')]/..";
        let any_img_xpath = "//div[contains(@style,'background-image: url(')]";
        let second_any_img_xpath = "//picture";
        let url_partial_xpath = "./a[contains(@href,'/rooms/')]";
        let img_partial_xpath = "./div[1]//div[contains(@style,'background-image: url(')]";
        let second_img_partial_xpath = "./div[1]//picture/source[1]";

        c.wait_for_find(Locator::XPath(listing_container_xpath))
            .await?;
        let res = c.find_all(Locator::XPath(listing_container_xpath)).await;

        if let Ok(v) = res {
            println!("The number of elements with link: {}", v.len());
            let elements = v.into_iter();
            for mut element in elements {
                let mut room = RoomListingItem::new();
                let url_href = Scraper::select_url(&mut element, url_partial_xpath).await?;
                let img_style =
                    Scraper::select_img(&mut element, img_partial_xpath, second_img_partial_xpath)
                        .await?;

                Scraper::extract_url(&mut room, url_href);
                Scraper::extract_img_url(&mut room, img_style);
                for config in xpath_config.iter() {
                    let mut result = "".to_string();
                    for xpath in config.xpath.iter() {
                        Self::scrape_text(&mut element, &mut result, &xpath).await;
                    }
                    if !result.is_empty() {
                        RoomListingContent::update_room_listing(
                            &mut room.content,
                            &config.item_type,
                            &result,
                        );
                    }
                }
                &result.push(room);
            }
        }
        Ok(c)
    }

    pub async fn scrape(url_path: &str) -> Result<Vec<RoomListingItem>, Box<dyn Error>> {
        let mut c = Scraper::setup().await;
        let mut result: Vec<RoomListingItem> = vec![];

        let mut c = match c {
            Ok(v) => v,
            Err(e) => {
                println!("ERROR: {:?}", e);
                return Ok(result);
            }
        };

        println!("STARTED CLIENT");
        let executed_result = Self::execute(&mut c, url_path, &mut result).await;
        match executed_result {
            Ok(_) => (),
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        };
        c.close().await?;
        Ok(result)
    }
}
