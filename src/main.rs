use actix_web::*;
use anyhow::Context;
use log::*;
use std::env;

mod chinook;

type Db = sqlx::sqlite::Sqlite;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    // let max_conn: usize = match std::env::var("CONN") {
    //     Ok(s) => s.parse().unwrap_or_else(|_| {
    //         error!("cannot parse env var CONN as integer: {}", s);
    //         3 * num_cpus::get()
    //     }),
    //     _ => 3 * num_cpus::get(),
    // };
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        // .after_connect(move |conn| {
        //     Box::pin(async move {
        //         use sqlx::Executor;
        //         conn.execute("PRAGMA synchronous = NORMAL;").await?;
        //         conn.execute("PRAGMA journal_mode = WAL;").await?;
        //         conn.execute("PRAGMA temp_store = 2;").await?;
        //         conn.execute("PRAGMA cache_size = -64000;").await?;
        //         conn.execute("PRAGMA foreign_keys = ON;").await?;
        //         Ok(())
        //     })
        // })
        // .max_connections(max_conn as u32)
        .connect_with(env::var("DATABASE_URL").context("DATABASE_URL")?.parse()?)
        .await?;
    let addr = env::var("SOCKETADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    info!("this web server is listening at http://{}", &addr);
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .configure(chinook::service)
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(&addr)
    .context(addr)?
    .run()
    .await
    .context("While running actix web server")?;
    Ok(())
}
