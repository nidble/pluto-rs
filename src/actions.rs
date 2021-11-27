
use std::{convert::Infallible, marker::{Sync, Send}};
use serde::{Serialize};
use rweb::{Rejection, Reply, get, hyper::StatusCode, post};
use crate::model::{Exchange, ExchangeRepo};
use log::{Level, log};

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
    let result = api.add_exchange(exchange).await.map_err(|err| format_error(err, 1003))?;

    let reply = rweb::reply::json(&serde_json::json!({"uuid": result}));

    Ok(rweb::reply::with_status(reply, rweb::http::StatusCode::CREATED))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let mut message = "";
    let mut internal_code: Option<u16> = None;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
    } else if let Some(error) = err.find::<ErrorMessage>() {
        code = StatusCode::BAD_REQUEST;
        message = &error.message;
        internal_code = error.internal_code; 
    } else if err.find::<rweb::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
    } else {
        log!(Level::Error, "{}", format!("Unhandled rejection: {:?}", err));
        code = StatusCode::INTERNAL_SERVER_ERROR;
    }

    let json = rweb::reply::json(&ErrorMessage {
        code: Some(code.as_u16()),
        message: message.into(),
        internal_code
    });

    Ok(rweb::reply::with_status(json, code))
}

#[cfg(test)]
mod tests {
    use rweb::Filter;
    use rweb::{http::StatusCode, test::request};
    use mockall::*;
    use mockall::predicate::*;
    use async_trait::async_trait;

    use uuid_::Uuid;
    use crate::actions::handle_rejection;
    use crate::model::Exchange;

    use super::super::model::ExchangeRepo;
    use super::{
        new_exchange,
    };

    mock! {
        pub PostgresExchangeRepo {
            fn add_exchange(&self) -> anyhow::Result<Uuid> {}
        }
    }

    #[async_trait]
    impl ExchangeRepo for MockPostgresExchangeRepo {
        async fn ping(&self) -> anyhow::Result<()> { todo!() }
        async fn add_exchange(&self, _exchange: Exchange) -> anyhow::Result<Uuid> { Ok(Uuid::default()) }
    }
    
    impl Clone for MockPostgresExchangeRepo {
        fn clone(&self) -> Self {
            MockPostgresExchangeRepo::new()
        }
    }


    #[tokio::test]
    async fn test_create_exchange() {
        let mut exchange_repo = MockPostgresExchangeRepo::new();
        exchange_repo.expect_add_exchange().returning(|| Ok(Uuid::default()));
        let api = new_exchange(exchange_repo);
        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amount": 123}"#;

        let resp = request()
            .method("POST")
            .body(body)
            .path("/exchanges")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::CREATED);

    }

    #[tokio::test]
    async fn test_reject_create_exchange() {
        let mut exchange_repo = MockPostgresExchangeRepo::new();
        exchange_repo.expect_add_exchange().returning(|| Ok(Uuid::default()));

        let api = new_exchange(exchange_repo).recover(handle_rejection);
        let body = r#"{"wrong": true}"#;

        let resp = request()
            .method("POST")
            .body(body)
            .path("/exchanges")

            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}