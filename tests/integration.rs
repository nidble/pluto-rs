#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use pluto_rs::init_routes;
    use rweb::test::request;

    #[tokio::test]
    async fn new_exchanges_should_return_201() {
        dotenv().ok();

    let routes = init_routes(1).await.unwrap();

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

        let routes = init_routes(1).await.unwrap();

        let res = request()
            .method("GET")
            .path("/healthz")
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 200, "POST works");
    }
}
