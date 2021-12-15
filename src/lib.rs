use std::convert::Infallible;

use rweb::{Filter, Reply};
use sqlx::{postgres::PgPoolOptions, Postgres, Pool};

pub mod actions;
pub mod api;
pub mod http_error;
pub mod model;
pub mod util;

pub async fn init_deps(pool_no: u32) -> anyhow::Result<Pool<Postgres>> {
    let database_url = std::env::var("DATABASE_URL")?; // this is fixed and by design for sqlx
    let pool = PgPoolOptions::new()
        .max_connections(pool_no)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn init_routes(pool: Pool<Postgres>) -> anyhow::Result<
    impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone,
> {
    let model = model::ExchangeRepository::new(pool);
    let api_service = api::Currency::new();
    let health_check = actions::status(model.clone());
    let exchanges = actions::new_exchange(model.clone(), api_service)
        .recover(actions::handle_rejection)
        .with(rweb::log("exchanges"));

    Ok(health_check.or(exchanges))
}
