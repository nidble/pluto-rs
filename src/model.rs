use chrono::{DateTime, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{postgres::PgPool, types::BigDecimal};
use uuid_::Uuid;

use crate::{actions::BodyData, util::get_datetime_zero};

pub struct Exchange {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub currency_from: String,
    pub currency_to: String,
    pub amount_from: BigDecimal,
    pub amount_to: BigDecimal,
}

trait ToBigDecimal {
    fn to_bigdecimal(&self) -> BigDecimal;
}

impl ToBigDecimal for f64 {
    fn to_bigdecimal(&self) -> BigDecimal {
        BigDecimal::from_f64(*self).unwrap_or_default()
    }
}

fn to_bigdecimal<F64: ToBigDecimal>(f: F64) -> BigDecimal{
    F64::to_bigdecimal(&f)
}
trait RoundTwo {
    fn round_two(&self) -> f64;
}

impl RoundTwo for BigDecimal {
    fn round_two(&self) -> f64 {
        let to = self.to_f64().unwrap_or_default();
        (to * 100.0).round() / 100.0
    }
}

fn round_two<R: RoundTwo>(value: &R) -> f64 {
    R::round_two(value)
}

impl Serialize for Exchange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Exchange", 6)?;
        let created_at = if cfg!(test) { get_datetime_zero() } else { self.created_at };

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
pub trait ExchangeRepo {
    async fn ping(&self) -> anyhow::Result<()>;

    async fn add_exchange(&self, body_data: BodyData, new_value: f64) -> anyhow::Result<Exchange>;
}

#[derive(Clone)]
pub struct PostgresExchangeRepo {
    pub pg_pool: Arc<PgPool>,
}

impl PostgresExchangeRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

#[async_trait]
impl ExchangeRepo for PostgresExchangeRepo {
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
    use std::str::FromStr;

    use serde_json;
    use sqlx::types::BigDecimal;

    use crate::model::{round_two, to_bigdecimal};

    use super::Exchange;

    #[test]
    fn test_exchange_serialize_works() {
        let exchange = Exchange::default();
        let body = r#"{"id":"00000000-0000-0000-0000-000000000000","created_at":"1970-01-01T00:00:00Z","currency_from":"","currency_to":"","amount_from":0.0,"amount_to":0.0}"#;
        assert_eq!(
            serde_json::to_string(&exchange).unwrap_or_default(),
            body.to_string()
        );
    }

    #[test]
    fn test_exchange_to_bigdecimal_works() {
        assert_eq!(
            to_bigdecimal(-123.0),
            BigDecimal::from_str("-123.0000000000000").unwrap()
        );
    }

    #[test]
    fn test_exchange_round_two_works() {
        assert_eq!(
            round_two(&BigDecimal::from_str("123.12").unwrap()),
            123.12
        );
    }
}
