use std::{env, error::Error, num::ParseFloatError, time::Duration};

use reqwest::StatusCode;
use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use serde::Serialize;
use sqlx::{prelude::FromRow, query, query_as, PgPool};

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Offer {
    pub id: Option<i32>,
    pub price: f32,
    pub detail_url: String,
    pub rooms: i16,
    pub area: f32,
    pub location: String,
    pub title: String,
    pub price_per_m2: Option<f32>,
}

async fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .user_agent("insomnia/8.6.1")
        .build()?;
    let html = client.get(url).send().await?.text().await?;
    Ok(html)
}

// fn get_html_from_file(file_path: &str) -> Result<String, Box<dyn Error>> {
//     let html = std::fs::read_to_string(file_path)?;
//     Ok(html)
// }

fn get_price_from_text(text: String) -> Result<f32, ParseFloatError> {
    // sample price:"419\u{a0}000\u{a0}zł", expected result: 419000
    let cleaned_price = text
        .replace(',', ".")
        .chars()
        .filter(|c| c.is_digit(10) || *c == '.')
        .collect::<String>();

    let price = cleaned_price.parse::<f32>();
    price
}

fn extract_title_from_element(element: &ElementRef) -> Result<String, Box<dyn Error>> {
    let title_selector = Selector::parse(r#"p[data-cy="listing-item-title"]"#).unwrap();
    let title = element
        .select(&title_selector)
        .next()
        .ok_or("No title found")?;
    let title = title.text().collect::<Vec<_>>().join(" ");
    Ok(title)
}

fn extract_price_from_element(element: &ElementRef) -> Result<f32, ParseFloatError> {
    let price_selector = Selector::parse(r#"div[data-testid="listing-item-header"] span"#).unwrap();
    let price = element.select(&price_selector).next().unwrap();
    let price_text = price.text().collect::<Vec<_>>().join(" ");
    get_price_from_text(price_text)
}

fn extract_detail_url_from_element(element: &ElementRef) -> Result<String, Box<dyn Error>> {
    let url_selector = Selector::parse(r#"a[data-testid="listing-item-link"]"#).unwrap();
    let url = element.select(&url_selector).next().ok_or("No url found")?;
    let url = url.value().attr("href").ok_or("No href attribute found")?;
    Ok(format!("https://www.otodom.pl{}", url))
}

fn extract_location_from_element(element: &ElementRef) -> Result<String, Box<dyn Error>> {
    let location_selector = Selector::parse(r#"p[data-testid="advert-card-address"]"#).unwrap();
    let location = element
        .select(&location_selector)
        .next()
        .ok_or("No location found")?;
    let location = location.text().collect::<Vec<_>>().join(" ");
    Ok(location)
}

fn extract_rooms_from_element(element: &ElementRef) -> Result<i16, Box<dyn Error>> {
    let dl_selector = Selector::parse(r#"div[data-testid="advert-card-specs-list"] > dl"#).unwrap();
    let dl_element = element
        .select(&dl_selector)
        .next()
        .ok_or("No dl element found")?;
    let dd_selector = Selector::parse("dd").unwrap();
    let dd_elements = dl_element.select(&dd_selector).collect::<Vec<_>>();
    let room_dd = dd_elements.get(0).ok_or("No first child found")?;
    let rooms_text = room_dd.text().collect::<Vec<_>>().join(" ");
    let rooms = rooms_text
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse::<i16>()?;
    Ok(rooms)
}

fn extract_area_from_element(element: &ElementRef) -> Result<f32, Box<dyn Error>> {
    let dl_selector = Selector::parse(r#"div[data-testid="advert-card-specs-list"] > dl"#).unwrap();
    let dl_element = element
        .select(&dl_selector)
        .next()
        .ok_or("No dl element found")?;
    let dd_selector = Selector::parse("dd").unwrap();
    let dd_elements = dl_element.select(&dd_selector).collect::<Vec<_>>();

    let area_dd = dd_elements.get(1).ok_or("No second dd element found")?;
    let text = area_dd.text().collect::<Vec<_>>().join(" ");
    let area = text
        .chars()
        .filter(|&c| c.is_digit(10) || c == '.')
        .collect::<String>()
        .parse::<f32>()?;
    Ok(area)
}

fn extract_last_page_number(element: &ElementRef) -> Result<i32, Box<dyn Error>> {
    let pagination_selector =
        Selector::parse(r#"ul[data-testid="frontend.search.base-pagination.nexus-pagination"]"#)
            .unwrap();
    let pagination = element
        .select(&pagination_selector)
        .next()
        .ok_or("No pagination found")?;
    // Last page number is second to last element
    let page_selector = Selector::parse("li").unwrap();
    let pages = pagination.select(&page_selector).collect::<Vec<_>>();
    let last_page = pages
        .get(pages.len() - 2)
        .ok_or("No second to last page found")?;
    let last_page = last_page.text().collect::<Vec<_>>().join(" ");
    let last_page = last_page
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse::<i32>()?;
    Ok(last_page)
}

fn get_price_per_m2(price: f32, area: f32) -> f32 {
    price / area
}

fn parse_article(article_element: &ElementRef) -> Result<Offer, Box<dyn Error>> {
    let price = extract_price_from_element(article_element);
    let detail_url = extract_detail_url_from_element(article_element);
    let location = extract_location_from_element(article_element);
    let rooms = extract_rooms_from_element(article_element);
    let area = extract_area_from_element(article_element);
    let title = extract_title_from_element(article_element);

    let price_per_m2 = match (&price, &area) {
        (Ok(price), Ok(area)) => Some(get_price_per_m2(*price, *area)),
        _ => None,
    };

    Ok(Offer {
        price: price?,
        detail_url: detail_url?,
        rooms: rooms?,
        area: area?,
        location: location?,
        title: title?,
        id: None,
        price_per_m2,
    })
}

async fn scrape_page(page_number: i32, pool: PgPool) -> Result<(), Box<dyn Error>> {
    let offers: Vec<Offer> = {
        let scrapping_url = env::var("SCRAPPING_URL").expect("SCRAPPING_URL must be set");
        let html = fetch_html(&format!("{}&page={}", scrapping_url, page_number)).await?;
        let document = Html::parse_document(html.as_str());
        let article_selector = Selector::parse(r#"article[data-cy="listing-item"]"#)?;
        let main_div_selector = Selector::parse("#__next")?;
        let main_div = document
            .select(&main_div_selector)
            .next()
            .ok_or("No main div found")?;
        let articles = main_div.select(&article_selector).collect::<Vec<_>>();
        articles
            .into_iter()
            .flat_map(|a| {
                parse_article(&a)
                    .inspect_err(|e| println!("Error parsing article: {:?}", e))
                    .ok()
            })
            .collect::<Vec<Offer>>()
    };
    let existing_offers: Vec<Offer> = query_as(r#"SELECT * FROM offers"#).fetch_all(&pool).await?;
    let new_offers = offers
        .clone()
        .into_iter()
        .filter(|offer| {
            !existing_offers
                .iter()
                .any(|existing_offer| existing_offer.detail_url == offer.detail_url)
        })
        .collect::<Vec<Offer>>();
    let offers_to_update = offers
        .clone()
        .into_iter()
        .filter(|offer| {
            existing_offers
                .iter()
                .any(|existing_offer| existing_offer.detail_url == offer.detail_url)
        })
        .collect::<Vec<Offer>>();

    for offer in new_offers {
        query!(
            r#"
        INSERT INTO offers (price, detail_url, rooms, area, location, title, price_per_m2)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
            offer.price,
            offer.detail_url,
            offer.rooms,
            offer.area,
            offer.location,
            offer.title,
            offer.price_per_m2
        )
        .execute(&pool)
        .await?;
        println!("Inserted offer: {:?}", offer);
    }

    for offer in offers_to_update {
        query!(
            r#"
        UPDATE offers
        SET price = $1, rooms = $2, area = $3, location = $4, title = $5, price_per_m2 = $6
        WHERE detail_url = $7
        "#,
            offer.price,
            offer.rooms,
            offer.area,
            offer.location,
            offer.title,
            offer.price_per_m2,
            offer.detail_url
        )
        .execute(&pool)
        .await?;
        println!("Updated offer: {:?}", offer);
    }
    Ok(())
}

pub async fn scrape_all_pages(timeout: Duration, pool: PgPool) -> Result<(), Box<dyn Error>> {
    let main_div_selector = Selector::parse("#__next")?;
    let last_page = {
        let scrapping_url = env::var("SCRAPPING_URL").expect("SCRAPPING_URL must be set");
        let html = fetch_html(&scrapping_url).await?;
        let document = Html::parse_document(html.as_str());
        let main_div = document
            .select(&main_div_selector)
            .next()
            .ok_or("No main div found")?;
        extract_last_page_number(&main_div)?
    };
    for page_number in 1..=last_page {
        scrape_page(page_number, pool.clone()).await?;
        tokio::time::sleep(timeout).await;
    }

    Ok(())
}

pub async fn clean_up_dead_offers(pool: PgPool) -> Result<(), Box<dyn Error>> {
    let offers: Vec<Offer> = query_as(r#"SELECT * FROM offers"#).fetch_all(&pool).await?;
    let detail_urls = offers
        .iter()
        .map(|offer| offer.detail_url.clone())
        .collect::<Vec<String>>();
    // fetch each offer and check if it's still available (status code 200)
    // if not, remove it from the database
    let client = reqwest::Client::builder()
        .user_agent("insomnia/8.6.1")
        .build()?;
    let responses =
        futures::future::join_all(detail_urls.iter().map(|url| client.get(url).send())).await;

    for (response, offer) in responses.iter().zip(offers.iter()) {
        match response {
            Ok(response) => {
                if response.status() == StatusCode::NOT_FOUND {
                    query!(
                        r#"DELETE FROM offers WHERE detail_url = $1"#,
                        offer.detail_url
                    )
                    .execute(&pool)
                    .await?;
                    println!("Deleted offer: {:?}", offer);
                }
            }
            Err(e) => println!("Error fetching offer: {:?}", e),
        };
    }

    Ok(())
}

fn save_offers_to_file(offers: Vec<Offer>) -> Result<(), Box<dyn Error>> {
    let offers_json = serde_json::to_string(&offers)?;
    std::fs::write("offers.json", offers_json)?;
    Ok(())
}
