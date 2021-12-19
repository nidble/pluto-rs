use std::convert::Infallible;

use rweb::{Filter, Reply};
use sqlx::postgres::PgPoolOptions;

use api::{Api, Currency};
use model::{ExchangeRepository, Repository};

pub mod actions;
pub mod api;
pub mod http_error;
pub mod model;
pub mod util;

pub async fn init_deps(pool_no: u32) -> anyhow::Result<(impl Repository, impl Api)> {
    let database_url = std::env::var("DATABASE_URL")?; // this is fixed and by design for sqlx
    let pool = PgPoolOptions::new()
        .max_connections(pool_no)
        .connect(&database_url)
        .await?;

    let model = ExchangeRepository::new(pool);
    let api_service = Currency::new();

    Ok((model, api_service))
}

pub fn init_routes<R, A>(
    repo: R,
    api_service: A,
) -> anyhow::Result<impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone>
where
    R: Repository,
    A: Api,
{
    let health_check = actions::status(repo.clone());
    let exchanges = actions::new_exchange(repo, api_service)
        .recover(actions::handle_rejection)
        .with(rweb::log("exchanges"));

    Ok(health_check.or(exchanges))
}
