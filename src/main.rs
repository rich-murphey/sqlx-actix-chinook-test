use actix_web::*;
use anyhow::Context;
use log::*;
use sqlx::{Pool, prelude::*, sqlite::*};
use std::env;

type Db = Sqlite;
type DbPool = Pool<Db>;

// mod websocket;
mod chinook;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let db_url = env::var("DATABASE_URL").context("DATABASE_URL")?;
    let max_conn: usize = match std::env::var("CONN") {
        Ok(s) => s.parse().unwrap_or_else(|_| {
            error!("cannot parse env var CONN as integer: {}", s);
            num_cpus::get()
        }),
        _ => num_cpus::get(),
    };
    let pool = {
        use std::time::Duration;
        let mut options: SqliteConnectOptions = db_url.parse()?;
        options
            .log_slow_statements(log::LevelFilter::Off, Duration::default())
            .log_statements(log::LevelFilter::Off);
        SqlitePoolOptions::new()
            .max_connections(max_conn as u32)
            .connect_with(options)
            .await?
    };
    let hostname = env::var("HOSTNAME")
        .unwrap_or_else(|_| sys_info::hostname().unwrap_or_else(|_| "localhost".to_string()));
    let addr = env::var("SOCKETADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    info!("this web server is listening at http://{}", &addr);
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .configure(chinook::service)
            // .configure(websocket::service)
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .server_hostname(&hostname)
    .shutdown_timeout(5) // docker waits 10 sec before killing.
    .workers(max_conn) // actix's default is = logical cpu count
    .bind(&addr)
    .context(addr)?
    .run()
    .await
    .context("While running actix web server")?;
    Ok(())
}
