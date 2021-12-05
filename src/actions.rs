use chrono::{DateTime, Utc};
use log::{log, Level};
use rweb::{post, Rejection, Reply};
use serde::Deserialize;
use serde_json::json;
use std::{
    convert::Infallible,
    marker::{Send, Sync},
};

use crate::http_error::{HttpError, format_error};
use crate::model::ExchangeRepo;
use crate::util;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyData {
    pub created_at: DateTime<Utc>,
    pub currency_from: String,
    pub currency_to: String,
    pub amount_from: f64,
}

#[post("/exchanges")]
pub async fn new_exchange(
    #[data] api: impl ExchangeRepo + Clone + Send + Sync,
    #[body] body: bytes::Bytes,
) -> Result<impl Reply, Rejection> {
    let json = std::str::from_utf8(&body).map_err(format_error(1001))?;
    let bd: BodyData = serde_json::from_str(json).map_err(format_error(1002))?;

    let amount = util::exchange(&bd.currency_from, &bd.currency_to, bd.amount_from)
        .map_err(format_error(1003))?;
    let exchange = api
        .add_exchange(bd, amount)
        .await
        .map_err(format_error(1004))?;

    let reply = rweb::reply::json(&exchange);

    Ok(rweb::reply::with_status(
        reply,
        rweb::http::StatusCode::CREATED,
    ))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, resp) = match HttpError::resolve_rejection(&err) {
        HttpError::NotFound(s)
        | HttpError::InternalServerError(s)
        | HttpError::MethodNotAllowed(s) => (s, json!({"message": s.canonical_reason() })),
        HttpError::BadRequest(s, e) => (
            s,
            json!({"message": e.message, "internalCode": e.internal_code }),
        ),
    };

    log!(
        Level::Error,
        "{}",
        format!("Unhandled rejection: {:?}", err)
    );
    Ok(rweb::reply::with_status(rweb::reply::json(&resp), code))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    use rweb::test::request;
    use rweb::Filter;
    use std::sync::{Arc, Mutex};

    use super::{handle_rejection, new_exchange, BodyData};
    use crate::model::{Exchange, ExchangeRepo};

    mock! {
        pub PostgresExchangeRepo {
            fn add_exchange(&self, body_data: BodyData, _new_value: f64) -> anyhow::Result<Exchange>;
        }
    }

    #[async_trait]
    impl ExchangeRepo for Arc<Mutex<MockPostgresExchangeRepo>> {
        async fn ping(&self) -> anyhow::Result<()> {
            todo!()
        }

        async fn add_exchange(
            &self,
            body_data: BodyData,
            new_value: f64,
        ) -> anyhow::Result<Exchange> {
            let this = self.lock().unwrap();
            this.add_exchange(body_data, new_value)
        }
    }

    fn get_repo_mock(times: TimesRange) -> Arc<Mutex<MockPostgresExchangeRepo>> {
        let mut repo = MockPostgresExchangeRepo::new();
        repo.expect_add_exchange()
            .times(times)
            .returning(|_, _| Ok(Exchange::default()));
        Arc::new(Mutex::new(repo))
    }

    #[tokio::test]
    async fn test_create_exchange() {
        let repo = get_repo_mock(1.into());
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
        let repo = get_repo_mock(0.into());
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
