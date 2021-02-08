use actix_web::*;
use serde::*;
use sqlx::prelude::*;
use sqlx_actix_streaming::*;

use super::{Db, DbPool};

#[derive(Serialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct TrackRec {
    pub track_id: i64,      // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub name: String,       // NVARCHAR(200)  NOT NULL,
    pub album_id: i64,      // INTEGER,
    pub media_type_id: i64, // INTEGER  NOT NULL,
    pub genre_id: i64,      // INTEGER,
    pub composer: String,   // NVARCHAR(220),
    pub milliseconds: i64,  // INTEGER  NOT NULL,
    pub bytes: i64,         // INTEGER,
    pub unit_price: f32,    // NUMERIC(10,2)  NOT NULL,
}

#[derive(Serialize, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct TrackRecRef<'a> {
    pub track_id: i64,      // INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pub name: &'a str,      // NVARCHAR(200)  NOT NULL,
    pub album_id: i64,      // INTEGER,
    pub media_type_id: i64, // INTEGER  NOT NULL,
    pub genre_id: i64,      // INTEGER,
    pub composer: &'a str,  // NVARCHAR(220),
    pub milliseconds: i64,  // INTEGER  NOT NULL,
    pub bytes: i64,         // INTEGER,
    pub unit_price: f32,    // NUMERIC(10,2)  NOT NULL,
}

#[derive(Deserialize, Serialize)]
pub struct TrackParams {
    pub offset: i64,
    pub limit: i64,
}

#[post("/tracks")]
pub async fn tracks(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ByteStream::make(
            RowWStmtStream::make(
                pool.as_ref(),
                "SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ",
                |pool, sql| {
                    sqlx::query_as::<Db,TrackRec>(sql)
                        .bind(params.limit)
                        .bind(params.offset)
                        .fetch(pool)
                },
            ),
            |buf: &mut BytesWriter, rec| {
                serde_json::to_writer( buf, rec).map_err(error::ErrorInternalServerError)
            },
        ))
}

#[post("/tracksref")]
pub async fn tracksref(
    web::Json(params): web::Json<TrackParams>,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ByteStream::make(
            RowWStmtStream::make(
                pool.as_ref(),
                "SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ",
                |pool, sql| {
                    sqlx::query(sql)
                        .bind(params.limit)
                        .bind(params.offset)
                        .fetch(pool)
                },
            ),
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
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let mut prefix = r#"{"params":"#.to_string();
    prefix.push_str(&serde_json::to_string(&params).unwrap());
    prefix.push_str(r#","data":["#);
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(
            query_byte_stream!(
                pool.as_ref(),
                "SELECT * FROM tracks LIMIT ?1 OFFSET ?2 ".to_string(),
                |buf: &mut BytesWriter, row| {
                    serde_json::to_writer(
                        buf,
                        &TrackRecRef::from_row(row).map_err(error::ErrorInternalServerError)?,
                    )
                    .map_err(error::ErrorInternalServerError)
                },
                params.limit,
                params.offset
            )
            .prefix(prefix)
            .suffix(r#"]}"#),
        )
}

pub fn service(cfg: &mut web::ServiceConfig) {
    cfg.service(tracks);
    cfg.service(tracksobj);
}
