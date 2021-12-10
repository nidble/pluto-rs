use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{postgres::PgPool, types::BigDecimal};
use uuid_::Uuid;

use crate::{
    actions::BodyData,
    util::{get_datetime_zero, round_two, to_bigdecimal},
};

pub struct Exchange {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub currency_from: String,
    pub currency_to: String,
    pub amount_from: BigDecimal,
    pub amount_to: BigDecimal,
}

impl Serialize for Exchange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Exchange", 6)?;
        let created_at = if cfg!(test) {
            get_datetime_zero()
        } else {
            self.created_at
        };

        state.serialize_field("id", &self.id)?;
        state.serialize_field("created_at", &created_at)?;
        state.serialize_field("currency_from", &self.currency_from)?;
        state.serialize_field("currency_to", &self.currency_to)?;
        state.serialize_field("amount_from", &round_two(&self.amount_from))?;
        state.serialize_field("amount_to", &round_two(&self.amount_to))?;
        state.end()
    }
}

impl Default for Exchange {
    fn default() -> Self {
        Self {
            id: Default::default(),
            created_at: chrono::Utc::now(),
            currency_from: Default::default(),
            currency_to: Default::default(),
            amount_from: Default::default(),
            amount_to: Default::default(),
        }
    }
}

#[async_trait]
pub trait ModelRepo {
    async fn ping(&self) -> anyhow::Result<()>;
    async fn add_exchange(&self, body_data: BodyData, new_value: f64) -> anyhow::Result<Exchange>;
}

#[derive(Clone)]
pub struct ExchangeRepo {
    pub pg_pool: Arc<PgPool>,
}

impl ExchangeRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

#[async_trait]
impl ModelRepo for ExchangeRepo {
    async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT $1")
            .bind(42)
            .fetch_one(&*self.pg_pool)
            .await?;

        Ok(())
    }

    async fn add_exchange(&self, body_data: BodyData, new_value: f64) -> anyhow::Result<Exchange> {
        let from = to_bigdecimal(body_data.amount_from);
        let to = to_bigdecimal(new_value);
        let exchange = sqlx::query_as!(Exchange,
            r#"
INSERT INTO exchanges ( amount_from, amount_to, currency_from, currency_to, created_at ) VALUES ( $1, $2, $3, $4, $5 )
RETURNING id, amount_from, amount_to, currency_from, currency_to, created_at
        "#,
            from, to, body_data.currency_from, body_data.currency_to, body_data.created_at
        )
        .fetch_one(&*self.pg_pool).await?;

        Ok(exchange)
    }
}

#[cfg(test)]
mod tests {
    use super::Exchange;
    use serde_json;

    #[test]
    fn test_exchange_serialize_works() {
        let exchange = Exchange::default();
        let body = r#"{"id":"00000000-0000-0000-0000-000000000000","created_at":"1970-01-01T00:00:00Z","currency_from":"","currency_to":"","amount_from":0.0,"amount_to":0.0}"#;
        assert_eq!(
            serde_json::to_string(&exchange).unwrap_or_default(),
            body.to_string()
        );
    }
}
