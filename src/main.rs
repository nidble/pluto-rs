use dotenv::dotenv;
use log::{log, Level};
use pluto_rs::init_routes;

mod actions;
mod api;
mod http_error;
mod model;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let routes = init_routes(5).await?;
    let routes = init_routes(5).await?;

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
