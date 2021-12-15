#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use pluto_rs::{init_deps, init_routes};
    use rweb::test::request;

    #[tokio::test]
    async fn new_exchanges_should_return_201() {
        dotenv().ok();

        let pool = init_deps(1).await.unwrap();
        let routes = init_routes(pool).unwrap();

        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amountFrom": 123, "createdAt": "2012-04-23T18:25:43.511Z"}"#;
        let res = request()
            .method("POST")
            .body(body)
            .path("/exchanges")
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 201, "POST works");
    }

    #[tokio::test]
    async fn healthcheck_should_return_200() {
        dotenv().ok();

        let pool = init_deps(1).await.unwrap();
        let routes = init_routes(pool).unwrap();

        let res = request()
            .method("GET")
            .path("/healthz")
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 200, "POST works");
    }
}
