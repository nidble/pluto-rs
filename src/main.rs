use dotenv::dotenv;
use log::{log, Level};
use pluto_rs::{init_routes, init_deps};

mod actions;
mod api;
mod http_error;
mod model;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let pool = init_deps(5).await?;
    let routes = init_routes(pool)?;

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
