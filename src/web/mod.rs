use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::{fs::File, io::Read, path::Path};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

mod util;

use util::*;

pub fn get_web_app(pool: PgPool) -> Router {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("./static/dist/assets"))
        .route("/", get(serve_index))
        .route("/health_check", get(health_check))
        .route("/offers", get(get_offers))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_origin("http://localhost:3001".parse::<HeaderValue>().unwrap())
                .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::DELETE, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(pool);

    app
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    let path = Path::new("./static/dist/index.html");
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Html(contents))
}

async fn handler() -> &'static str {
    "Hello, World!"
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Deserialize, Debug, Clone)]
pub struct OfferParams {
    id: Option<i32>,
    rooms: Option<i16>,
    price: Option<f32>,
    location: Option<String>,
    exclude: Option<bool>,
}

impl Default for OfferParams {
    fn default() -> Self {
        OfferParams {
            id: None,
            rooms: None,
            price: None,
            location: None,
            exclude: None,
        }
    }
}

async fn get_offers(
    State(pool): State<PgPool>,
    Query(params): Query<OfferParams>,
) -> impl IntoResponse {
    let offers = filter_offers(params.clone(), pool).await;

    match offers {
        Ok(offers) => Json(offers),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            Json(vec![])
        }
    }
}
