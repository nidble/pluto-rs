use chrono::{DateTime, Utc};
use log::{log, Level};
use rweb::{get, post, Rejection, Reply};
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;

use crate::api;
use crate::http_error::{decorate_error, HttpError};
use crate::model;
use crate::util;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyData {
    pub created_at: DateTime<Utc>,
    pub currency_from: String,
    pub currency_to: String,
    pub amount_from: f64,
}

#[get("/healthz")]
pub async fn status(#[data] repo: impl model::Repository) -> Result<impl Reply, Rejection> {
    repo.ping().await.map_err(decorate_error(1001))?;

    Ok(rweb::reply::reply())
}

#[post("/api/v1/exchanges")]
pub async fn new_exchange(
    #[data] repo: impl model::Repository,
    #[data] api: impl api::Api,
    #[body] body: bytes::Bytes,
) -> Result<impl Reply, Rejection> {
    let json = std::str::from_utf8(&body).map_err(decorate_error(1020))?;
    let bd: BodyData = serde_json::from_str(json).map_err(decorate_error(1030))?;

    let rate = api
        .get_rate(&bd.currency_from, &bd.currency_to, "latest")
        .await
        .map_err(decorate_error(1040))?;

    let amount = util::exchange(&bd.currency_from, &bd.currency_to, bd.amount_from, rate)
        .map_err(decorate_error(1050))?;

    let exchange = repo
        .add_exchange(bd, amount)
        .await
        .map_err(decorate_error(1060))?;

    let reply = rweb::reply::json(&exchange);

    Ok(rweb::reply::with_status(
        reply,
        rweb::http::StatusCode::CREATED,
    ))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, resp) = match HttpError::resolve_rejection(&err) {
        HttpError::NotFound(status_code)
        | HttpError::InternalServerError(status_code)
        | HttpError::MethodNotAllowed(status_code) => (
            status_code,
            json!({"message": status_code.canonical_reason() }),
        ),
        HttpError::BadRequest(status_code, e) => (
            status_code,
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
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use rweb::test::request;
    use std::sync::{Arc, Mutex};

    use super::{new_exchange, BodyData};
    use crate::{
        api::Api,
        model::{Exchange, Repository},
    };

    mock! {
        pub ExchangeRepository {
            fn add_exchange(&self, body_data: BodyData, _new_value: f64) -> anyhow::Result<Exchange>;
        }
    }

    mock! {
        pub Api {
            fn get_rate(&self, from: &str, to: &str, date: &str) -> anyhow::Result<Decimal>;
        }
    }

    #[async_trait]
    impl Repository for Arc<Mutex<MockExchangeRepository>> {
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

    #[async_trait]
    impl Api for Arc<Mutex<MockApi>> {
        async fn get_rate(&self, from: &str, to: &str, date: &str) -> anyhow::Result<Decimal> {
            let this = self.lock().unwrap();
            this.get_rate(from, to, date)
        }
    }

    fn get_repo_mock(times: TimesRange) -> Arc<Mutex<MockExchangeRepository>> {
        let mut repo = MockExchangeRepository::new();
        repo.expect_add_exchange()
            .times(times)
            .returning(|_, _| Ok(Exchange::default()));
        Arc::new(Mutex::new(repo))
    }

    fn get_api_mock(times: TimesRange) -> Arc<Mutex<MockApi>> {
        let mut api = MockApi::new();
        api.expect_get_rate()
            .times(times)
            .returning(|_, _, _| Ok(dec!(42)));
        Arc::new(Mutex::new(api))
    }

    #[tokio::test]
    async fn test_create_exchange() {
        let db_repo = get_repo_mock(1.into());
        let api = get_api_mock(1.into());
        let action = new_exchange(db_repo.clone(), api.clone());
        let body = r#"{"currencyFrom": "EUR", "currencyTo": "USD", "amountFrom": 123, "createdAt": "2012-04-23T18:25:43.511Z"}"#;

        request()
            .method("POST")
            .body(body)
            .path("/api/v1/exchanges")
            .reply(&action)
            .await;

        let mut db_repo = db_repo.lock().unwrap();
        db_repo.checkpoint();
        let mut api = api.lock().unwrap();
        api.checkpoint();
    }

    #[tokio::test]
    async fn test_reject_create_exchange() {
        let db_repo = get_repo_mock(0.into());
        let api = get_api_mock(0.into());
        let action = new_exchange(db_repo.clone(), api.clone());
        let body = r#"{"wrong": true}"#;

        request()
            .method("POST")
            .body(body)
            .path("/api/v1/exchanges")
            .reply(&action)
            .await;

        let mut db_repo = db_repo.lock().unwrap();
        db_repo.checkpoint();
        let mut api = api.lock().unwrap();
        api.checkpoint();
    }
}
