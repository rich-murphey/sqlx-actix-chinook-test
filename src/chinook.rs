use actix_web::*;
use serde::*;
use sqlx::{prelude::*, Pool};
use sqlx_actix_streaming::*;

use super::Db;

#[derive(Serialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct TrackRec {
    pub track_id: i64,            // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub name: String,             // NVARCHAR(200)  NOT NULL,
    pub album_id: Option<i64>,    // INTEGER,
    pub media_type_id: i64,       // INTEGER  NOT NULL,
    pub genre_id: Option<i64>,    // INTEGER,
    pub composer: Option<String>, // NVARCHAR(220),
    pub milliseconds: i64,        // INTEGER  NOT NULL,
    pub bytes: Option<i64>,       // INTEGER,
    pub unit_price: f32,          // REAL  NOT NULL,
}

#[derive(Serialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct TrackRecRef<'a> {
    pub track_id: i64,             // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub name: Option<&'a str>,     // NVARCHAR(200)  NOT NULL,
    pub album_id: Option<i64>,     // INTEGER,
    pub media_type_id: i64,        // INTEGER  NOT NULL,
    pub genre_id: Option<i64>,     // INTEGER,
    pub composer: Option<&'a str>, // NVARCHAR(220),
    pub milliseconds: i64,         // INTEGER  NOT NULL,
    pub bytes: Option<i64>,        // INTEGER,
    pub unit_price: f32,           // NUMERIC(10,2)  NOT NULL,
}

#[derive(Deserialize, Serialize)]
pub struct TrackParams {
    pub offset: i64,
    pub limit: i64,
}

#[post("/tracks")]
pub async fn tracks(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ByteStream::new(
            pool.as_ref(),
            move |pool| {
                sqlx::query_as::<_, TrackRec>("SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ")
                    .bind(params.limit)
                    .bind(params.offset)
                    .fetch(pool)
            },
            |buf: &mut BytesWriter, record: &TrackRec| {
                serde_json::to_writer(buf, record).map_err(error::ErrorInternalServerError)
            },
        ))
}

// this is the same as /tracks, except using a macro.
#[post("/tracks2")]
pub async fn tracks2(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
    json_response!(
        TrackRec,
        pool.as_ref(),
        "SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ",
        params.limit,
        params.offset
    )
}

#[post("/tracksref")]
pub async fn tracksref(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<Pool<Db>>,
) -> HttpResponse {
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
    cfg.service(tracks2);
    cfg.service(tracksref);
    cfg.service(tracksobj);
}
