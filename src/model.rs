use serde::{Serialize, Deserialize};
use rweb::Schema;
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use sqlx::postgres::types::PgMoney;
use uuid_::Uuid;

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    currency_from: String,
    currency_to: String,
    amount: i64,
}

#[async_trait]
pub trait ExchangeRepo {
    async fn ping(&self) -> anyhow::Result<()>;

    async fn add_exchange(&self, exchange: Exchange) -> anyhow::Result<Uuid>;
}

#[derive(Clone)]
pub struct PostgresExchangeRepo {
    pub pg_pool: PgPool,
}

impl PostgresExchangeRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }
}

#[async_trait]
impl ExchangeRepo for PostgresExchangeRepo {
    async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT $1").bind(42).fetch_one(&self.pg_pool.clone()).await?;

        Ok(())
    }

    async fn add_exchange(&self, e: Exchange) -> anyhow::Result<Uuid> {
        let rec = sqlx::query!(
            r#"
INSERT INTO exchanges ( amount_from, amount_to, currency_from, currency_to ) VALUES ( $1, $2, $3, $4 )
RETURNING id
        "#,
            PgMoney::from(e.amount), PgMoney::from(e.amount), e.currency_from, e.currency_to 
        )
        .fetch_one(&self.pg_pool.clone()).await?;

        Ok(rec.id)
    }
}