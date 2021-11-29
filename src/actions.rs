use std::{convert::Infallible, marker::{Sync, Send}};
use serde::Serialize;
use rweb::{Rejection, Reply, hyper::StatusCode, post};
use serde_json::json;
use log::{Level, log};

use crate::model::{Exchange, ExchangeRepo};
use crate::util;
/// An API error serializable to JSON.
#[derive(Serialize, Debug)]
struct ErrorMessage {
    code: Option<u16>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    internal_code: Option<u16>,
}
impl rweb::reject::Reject for ErrorMessage {}


fn format_error<E: std::fmt::Display>(err: E, internal_code: u16) -> Rejection {
    let error = ErrorMessage {
        internal_code: Some(internal_code),
        message: err.to_string(),
        code: None
    };
    rweb::reject::custom(error)
}

#[post("/exchanges")]
pub async fn new_exchange(#[data] api: impl ExchangeRepo + Clone + Send + Sync, #[body] body: bytes::Bytes) -> Result<impl Reply, Rejection> {
    let json = std::str::from_utf8(&body).map_err(|err| format_error(err, 1001))?;
    let exchange: Exchange = serde_json::from_str(&json).map_err(|err| format_error(err, 1002))?;

    let amount = util::exchange(&exchange.currency_from, &exchange.currency_to, exchange.amount).map_err(|err| format_error(err, 1003))?;
    let result = api.add_exchange(exchange, amount).await.map_err(|err| format_error(err, 1004))?;

    let reply = rweb::reply::json(&json!({"uuid": result}));

    Ok(rweb::reply::with_status(reply, rweb::http::StatusCode::CREATED))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, response) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, json!({"message": "not found"}))
    } else if let Some(error) = err.find::<ErrorMessage>() {
        (StatusCode::BAD_REQUEST, json!({"message": &error.message, "internalCode": error.internal_code}))
    } else if err.find::<rweb::reject::MethodNotAllowed>().is_some() {
        (StatusCode::METHOD_NOT_ALLOWED, json!({"message": "method not allowed"}))
    } else {
        log!(Level::Error, "{}", format!("Unhandled rejection: {:?}", err));
        (StatusCode::INTERNAL_SERVER_ERROR, json!({"message": "internal server error"}))
    };

    Ok(rweb::reply::with_status(rweb::reply::json(&response), code))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use rweb::Filter;
    use rweb::test::request;
    use mockall::*;
    use mockall::predicate::*;
    use async_trait::async_trait;

    use uuid_::Uuid;
    use crate::model::Exchange;

    use super::super::model::ExchangeRepo;
    use super::{handle_rejection, new_exchange};

    mock! {
        pub PostgresExchangeRepo {
            fn add_exchange(&self, _exchange: Exchange, _new_value: i64) -> anyhow::Result<Uuid>;
        }
    }

    #[async_trait]
    impl ExchangeRepo for Arc<Mutex<MockPostgresExchangeRepo>> {
        async fn ping(&self) -> anyhow::Result<()> { todo!() }
        async fn add_exchange(&self, _exchange: Exchange, _new_value: i64) -> anyhow::Result<Uuid> { Ok(Uuid::default()) }
        async fn fetch_exchanges(&self) -> anyhow::Result<()> { Ok(()) }
    }

    #[tokio::test]
    async fn test_create_exchange() {
        let mut repo = MockPostgresExchangeRepo::new();
        repo.expect_add_exchange()
            .returning(|_, _| Ok(Uuid::default()));
        let repo = Arc::new(Mutex::new(repo));
        let api = new_exchange(repo.clone());
        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amount": 123}"#;

        let res = request()
            .method("POST")
            .body(body)
            .path("/exchanges")
            .reply(&api)
            .await;

        let mut repo = repo.lock().unwrap();
        repo.checkpoint();
        assert_eq!(res.status(), 201, "POST works with 201");
    }

    #[tokio::test]
    async fn test_reject_create_exchange() {
        let mut repo = MockPostgresExchangeRepo::new();
        repo.expect_add_exchange().times(0).returning(|_, _| Ok(Uuid::default()));
        let repo = Arc::new(Mutex::new(repo));
        let api = new_exchange(repo.clone()).recover(handle_rejection);
        let body = r#"{"wrong": true}"#;

        let res = request()
            .method("POST")
            .body(body)
            .path("/exchanges")
            .reply(&api)
            .await;

        let mut repo = repo.lock().unwrap();
        repo.checkpoint();    
        assert_eq!(res.status(), 400, "POST works with 400");
    }
}