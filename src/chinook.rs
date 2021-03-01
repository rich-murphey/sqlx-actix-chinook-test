#![allow(non_snake_case)]
use actix_web::*;
use serde::*;
use sqlx::{prelude::*, Pool};
use sqlx_actix_streaming::*;
use validator::Validate;

use super::Db;

// see: https://github.com/Keats/validator/blob/master/README.md
macro_rules! bail_if_invalid [
    ( $params:ident ) => ({
        if ( $params ).validate().is_err() {
            return HttpResponse::BadRequest().finish();
        }
    });
];

#[derive(Serialize, FromRow)]
pub struct TrackRec {
    pub TrackId: i64,             // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub Name: String,             // NVARCHAR(200)  NOT NULL,
    pub AlbumId: Option<i64>,     // INTEGER,
    pub MediaTypeId: i64,         // INTEGER  NOT NULL,
    pub GenreId: Option<i64>,     // INTEGER,
    pub Composer: Option<String>, // NVARCHAR(220),
    pub Milliseconds: i64,        // INTEGER  NOT NULL,
    pub Bytes: Option<i64>,       // INTEGER,
    pub UnitPrice: f32,           // REAL  NOT NULL,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct TrackParams {
    pub offset: i64,
    #[validate(range(min = 1))]
    pub limit: i64,
}

#[post("/tracks")]
pub async fn tracks(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    bail_if_invalid!(params);
    json_response!(
        pool.as_ref().clone(),
        params,
        sqlx::query_as!(
            TrackRec,
            "select * from tracks limit ?1 offset ?2",
            params.limit,
            params.offset
        )
    )
}

#[get("/tracks4/{limit}/{offset}")]
pub async fn tracks4(path: web::Path<(i64, i64)>, pool: web::Data<Pool<Db>>) -> HttpResponse {
    let (limit, offset) = path.into_inner();
    sqlx_actix_streaming::query_json!(
        "select * from tracks limit ?1 offset ?2",
        pool.as_ref().clone(),
        limit,
        offset
    )
}

#[post("/tracks2")]
pub async fn tracks2(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(
        &sqlx::query!(
            "select * from tracks limit ?1 offset ?2",
            params.limit,
            params.offset
        )
        .fetch_all(pool.as_ref())
        .await
        .map_err(error::ErrorInternalServerError)?,
    ))
}

#[get("/tracks3/{limit}/{offset}")]
pub async fn tracks3(path: web::Path<(i64, i64)>, pool: web::Data<Pool<Db>>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ByteStream::new(
            RowStream::build(
                (pool.as_ref().clone(), path.into_inner()),
                move |(pool, (limit, offset))| {
                    sqlx::query!(
                        "select TrackId, Name, Composer, UnitPrice from tracks limit $1 offset $2",
                        *limit,
                        *offset,
                    )
                    .fetch(pool)
                },
            ),
            |buf: &mut BytesWriter, rec| {
                serde_json::to_writer(buf, rec).map_err(actix_web::error::ErrorInternalServerError)
            },
        ))
}

const UNKNOWN: &str = "(unknown)";

#[post("/track_table")]
pub async fn track_table(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(
            ByteStream::new(
                RowStream::build((pool.as_ref().clone(), params), move |(pool, params)| {
                    sqlx::query!(
                        "select TrackId, Name, Composer, UnitPrice from tracks limit $1 offset $2",
                        params.limit,
                        params.offset
                    )
                    .fetch(pool)
                }),
                |buf: &mut BytesWriter, rec| {
                    write!(
                        &mut *buf,
                        r#"[{}, {}, "{}", "{}"]"#,
                        rec.TrackId,
                        &rec.Name,
                        rec.Composer.as_ref().map_or(UNKNOWN, |s| s.as_str()),
                        rec.UnitPrice,
                    )
                    .map_err(error::ErrorInternalServerError)
                },
            )
            .prefix(r#"{"cols":["Track Id", "Name", "Composer", "Unit Price"],"rows":["#)
            .suffix(r#"]}"#),
        )
}

pub fn service(cfg: &mut web::ServiceConfig) {
    cfg.service(tracks);
    cfg.service(tracks2);
    cfg.service(tracks3);
    cfg.service(tracks4);
    cfg.service(track_table);
}
