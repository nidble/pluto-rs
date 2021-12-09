use pluto_rs::actions;
use pluto_rs::model;

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use pluto_rs::api::GithubRepo;
    use rweb::test::request;
    use sqlx::postgres::PgPoolOptions;

    use super::actions::new_exchange;
    use super::model::ExchangeRepo;

    #[tokio::test]
    async fn should_return_200() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap();

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap();

        let api = new_exchange(ExchangeRepo::new(pool).clone(), GithubRepo::new());
        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amountFrom": 123, "createdAt": "2012-04-23T18:25:43.511Z"}"#;
        let res = request()
            .method("POST")
            .body(body)
            .path("/exchanges")
            .reply(&api)
            .await;

        assert_eq!(res.status(), 201, "POST works");
    }
}
