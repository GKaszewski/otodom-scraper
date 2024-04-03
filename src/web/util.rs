use crate::scrapper::Offer;
use sqlx::{Execute, FromRow, PgPool, QueryBuilder};

use super::OfferParams;

pub async fn filter_offers(params: OfferParams, pool: PgPool) -> Result<Vec<Offer>, sqlx::Error> {
    let params = params.clone();
    let mut args = 0;

    let mut builder = QueryBuilder::new("SELECT * FROM offers");
    if let Some(id) = params.id {
        builder.push("WHERE id = ");
        builder.push_bind(id);
        args += 1;
    }
    if let Some(rooms) = params.rooms {
        if args == 0 {
            builder.push(" WHERE rooms = ");
            builder.push_bind(rooms);
        } else {
            builder.push(" AND rooms = ");
            builder.push_bind(rooms);
        }
        args += 1;
    }
    if let Some(price) = params.price {
        if args == 0 {
            builder.push(" WHERE price <= ");
            builder.push_bind(price);
        } else {
            builder.push(" AND price <= ");
            builder.push_bind(price);
        }
        args += 1;
    }
    let location = params.location.clone();
    if let Some(exclude) = params.exclude {
        if let Some(location) = location.clone() {
            if exclude {
                if args == 0 {
                    builder.push(" WHERE location NOT LIKE ");
                    builder.push_bind(format!("%{}%", location));
                } else {
                    builder.push(" AND location NOT LIKE ");
                    builder.push_bind(format!("%{}%", location));
                }
                args += 1;
            } else {
                if args == 0 {
                    builder.push(" WHERE location LIKE ");
                    builder.push_bind(format!("%{}%", location));
                } else {
                    builder.push(" AND location LIKE ");
                    builder.push_bind(format!("%{}%", location));
                }
                args += 1;
            }
        }
    }

    if let Some(location) = location {
        if params.exclude.is_none() {
            if args == 0 {
                builder.push(" WHERE location LIKE ");
                builder.push_bind(format!("%{}%", location));
            } else {
                builder.push(" AND location LIKE ");
                builder.push_bind(format!("%{}%", location));
            }
        }
    }

    builder.push(" ORDER BY price DESC;");

    let rows = builder.build().fetch_all(&pool).await?;
    let offers: Vec<Offer> = rows
        .iter()
        .map(|row| Offer::from_row(row).unwrap())
        .collect();

    Ok(offers)
}
