use dotenv::dotenv;
use log::{log, Level};
use rweb::Filter;
use sqlx::postgres::PgPoolOptions;

mod actions;
mod api;
mod http_error;
mod model;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let model = model::ExchangeRepository::new(pool);
    let api_service = api::Currency::new();

    let health_check = actions::status(model.clone());
    let exchanges = actions::new_exchange(model.clone(), api_service)
        .recover(actions::handle_rejection)
        .with(rweb::log("exchanges"));
    let routes = health_check.or(exchanges);

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
