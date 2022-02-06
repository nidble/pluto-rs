use dotenv::dotenv;
use log::{log, Level};
use pluto_rs::{init_deps, init_routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    let (repo, api_services) = init_deps(5).await?;
    let routes = init_routes(repo, api_services)?;

    log!(Level::Info, "Start up the server...");
    rweb::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
