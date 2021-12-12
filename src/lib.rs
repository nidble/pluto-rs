use std::convert::Infallible;

use rweb::{Filter, Reply};
use sqlx::postgres::PgPoolOptions;

pub mod actions;
pub mod api;
pub mod http_error;
pub mod model;
pub mod util;


pub async fn init_routes(pool_no: u32) -> anyhow::Result<
    impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone,
> {
    let database_url = std::env::var("DATABASE_URL")?; // this is by design for sqlx
    let pool = PgPoolOptions::new()
        .max_connections(pool_no)
        .connect(&database_url)
        .await?;

    let model = model::ExchangeRepository::new(pool);
    let api_service = api::Currency::new();
    let health_check = actions::status(model.clone());
    let exchanges = actions::new_exchange(model.clone(), api_service)
        .recover(actions::handle_rejection)
        .with(rweb::log("exchanges"));

    Ok(health_check.or(exchanges))
}
