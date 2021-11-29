use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use rweb::Filter;
use log::{Level, log};

mod model;
mod actions;
mod util;

/// Provides a RESTful web server managing Exchange.
///
/// API will be:
///
/// - `GET /exchanges`: return a JSON list of exchanges.
/// - `POST /exchanges`: create a new Exchange.
#[tokio::main]
async fn main()-> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await?;

    let api = actions::new_exchange(
        model::PostgresExchangeRepo::new(pool)
    ).recover(actions::handle_rejection);
    let routes = api.with(rweb::log("exchanges"));

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
