use std::{convert::Infallible, marker::{Sync, Send}};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use rweb::{Rejection, Reply, post};
use serde_json::json;
use log::{Level, log};

use crate::model::ExchangeRepo;
use crate::http_error::{HttpError, ErrorMessage};
use crate::util;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyData {
    pub created_at: DateTime<Utc>,
    pub currency_from: String,
    pub currency_to: String,
    pub amount_from: f64,
}

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
    let bd: BodyData = serde_json::from_str(&json).map_err(|err| format_error(err, 1002))?;

    let amount = util::exchange(&bd.currency_from, &bd.currency_to, bd.amount_from).map_err(|err| format_error(err, 1003))?;
    let resp = api.add_exchange(bd, amount).await.map_err(|err| format_error(err, 1004))?;

    let reply = rweb::reply::json(&resp);

    Ok(rweb::reply::with_status(reply, rweb::http::StatusCode::CREATED))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, resp) = match HttpError::resolve_rejection(&err) {
        HttpError::NotFound(s) |
        HttpError::InternalServerError(s) |
        HttpError::MethodNotAllowed(s)  => (s, json!({"message": s.canonical_reason() })),
        HttpError::BadRequest(s, e) => (s, json!({"message": e.message, "internalCode": e.internal_code })),
    };

    log!(Level::Error, "{}", format!("Unhandled rejection: {:?}", err));
    Ok(rweb::reply::with_status(rweb::reply::json(&resp), code))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use rweb::Filter;
    use rweb::test::request;
    use mockall::*;
    use mockall::predicate::*;
    use async_trait::async_trait;

    use crate::model::{ExchangeRepo, Exchange};
    use super::{BodyData, handle_rejection, new_exchange};

    mock! {
        pub PostgresExchangeRepo {
            fn add_exchange(&self, body_data: BodyData, _new_value: f64) -> anyhow::Result<Exchange>;
        }
    }

    #[async_trait]
    impl ExchangeRepo for Arc<Mutex<MockPostgresExchangeRepo>> {
        async fn ping(&self) -> anyhow::Result<()> { todo!() }

        async fn add_exchange(&self, body_data: BodyData, new_value: f64) -> anyhow::Result<Exchange> {
            let this = self.lock().unwrap();
            this.add_exchange(body_data, new_value)
         }

    }

    #[tokio::test]
    async fn test_create_exchange() {
        let mut repo = MockPostgresExchangeRepo::new();
        repo.expect_add_exchange()
            .times(1)
            .returning(|_, _| Ok(Exchange::default()));
        let repo = Arc::new(Mutex::new(repo));
        let api = new_exchange(repo.clone());
        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amountFrom": 123, "createdAt": "2012-04-23T18:25:43.511Z"}"#;

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
        repo.expect_add_exchange().times(0).returning(|_, _| Ok(Exchange::default()));
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