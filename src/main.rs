use dotenv::dotenv;
use log::{log, Level};
use rweb::Filter;
use sqlx::postgres::PgPoolOptions;

mod api;
mod actions;
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

    let api =
        actions::new_exchange(
            model::ExchangeRepo::new(pool),
            api::GithubRepo::new()
        ).recover(actions::handle_rejection);
    let routes = api.with(rweb::log("exchanges"));

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
