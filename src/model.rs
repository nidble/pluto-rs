use std::sync::Arc;
use serde::{Serialize, Deserialize};
use rweb::Schema;
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use sqlx::postgres::types::PgMoney;
use uuid_::Uuid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Schema)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub currency_from: String,
    pub currency_to: String,
    pub amount: i64,
}
#[async_trait]
pub trait ExchangeRepo {
    // fn get(&self) -> Arc<PgPool>;

    async fn ping(&self) -> anyhow::Result<()>;

    async fn add_exchange(&self, exchange: Exchange, new_value: i64) -> anyhow::Result<Uuid>;

    async fn fetch_exchanges(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct PostgresExchangeRepo {
    pub pg_pool: Arc<PgPool>,
}

impl PostgresExchangeRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool: Arc::new(pg_pool) }
    }
}

#[async_trait]
impl ExchangeRepo for PostgresExchangeRepo {
    async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT $1").bind(42).fetch_one(&*self.pg_pool).await?;

        Ok(())
    }

    async fn add_exchange(&self, e: Exchange, new_value: i64) -> anyhow::Result<Uuid> {
        let rec = sqlx::query!(
            r#"
INSERT INTO exchanges ( amount_from, amount_to, currency_from, currency_to ) VALUES ( $1, $2, $3, $4 )
RETURNING id
        "#,
            PgMoney::from(e.amount), PgMoney::from(new_value), e.currency_from, e.currency_to 
        )
        .fetch_one(&*self.pg_pool).await?;

        Ok(rec.id)
    }

    async fn fetch_exchanges(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
