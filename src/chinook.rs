#![allow(non_snake_case)]
use actix_web::*;
use serde::*;
use sqlx::{prelude::*, Pool};
use sqlx_actix_streaming::*;
use validator::Validate;

use super::Db;

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
    if params.validate().is_err() {
        return HttpResponse::BadRequest().finish();
    }
    json_response!(
        pool.as_ref().clone(),
        params,
        sqlx::query_as!(
            TrackRec,
            "SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ",
            params.limit,
            params.offset
        )
    )
}

#[derive(Serialize, FromRow)]
pub struct TrackRecRef<'a> {
    pub TrackId: i64,              // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub Name: &'a str,             // NVARCHAR(200)  NOT NULL,
    pub AlbumId: Option<i64>,      // INTEGER,
    pub MediaTypeId: i64,          // INTEGER  NOT NULL,
    pub GenreId: Option<i64>,      // INTEGER,
    pub Composer: Option<&'a str>, // NVARCHAR(220),
    pub Milliseconds: i64,         // INTEGER  NOT NULL,
    pub Bytes: Option<i64>,        // INTEGER,
    pub UnitPrice: f32,            // REAL  NOT NULL,
}

#[post("/tracksref")]
pub async fn tracksref(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    if params.validate().is_err() {
        return HttpResponse::BadRequest().finish();
    }
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ByteStream::new(
            pool.as_ref(),
            |pool| {
                sqlx::query("SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ")
                    .bind(params.limit)
                    .bind(params.offset)
                    .fetch(pool)
            },
            |buf: &mut BytesWriter, row| {
                serde_json::to_writer(
                    buf,
                    &TrackRecRef::from_row(row).map_err(error::ErrorInternalServerError)?,
                )
                .map_err(error::ErrorInternalServerError)
            },
        ))
}

#[post("/tracksobj")]
pub async fn tracksobj(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    if params.validate().is_err() {
        return HttpResponse::BadRequest().finish();
    }
    let mut prefix = r#"{"params":"#.to_string();
    prefix.push_str(&serde_json::to_string(&params).unwrap());
    prefix.push_str(r#","data":["#);
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(
            ByteStream::new(
                pool.as_ref(),
                move |pool| {
                    sqlx::query_as::<_, TrackRec>("SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ")
                        .bind(params.limit)
                        .bind(params.offset)
                        .fetch(pool)
                },
                |buf: &mut BytesWriter, rec: &TrackRec| {
                    serde_json::to_writer(buf, rec).map_err(error::ErrorInternalServerError)
                },
            )
            .prefix(prefix)
            .suffix(r#"]}"#),
        )
}

pub fn service(cfg: &mut web::ServiceConfig) {
    cfg.service(tracks);
    cfg.service(tracksref);
    cfg.service(tracksobj);
}
