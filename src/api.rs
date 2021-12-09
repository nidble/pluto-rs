use std::{collections::HashMap, fmt, error};
use async_trait::async_trait;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::Value;

// https://raw.githubusercontent.com/fawazahmed0/currency-api/1/latest/currencies/usd/eur.json { "date": "2021-12-07", "eur": 0.88645 }
static BASE_URL: &str = "https://raw.githubusercontent.com/fawazahmed0/currency-api/1";

#[derive(Debug, PartialEq)]
pub enum ApiError {
    RateNotAvailable,
    InvalidAmount,
    InvalidRatio,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ApiError::RateNotAvailable => write!(f, "Conversion rate non available"),
            ApiError::InvalidAmount => write!(f, "Conversion rate not parseable"),
            ApiError::InvalidRatio => write!(f, "Conversion rate was not valid"),
        }
    }
}

impl error::Error for ApiError {
    fn description(&self) -> &str {
        match *self {
            ApiError::RateNotAvailable => "Conversion rate non available",
            ApiError::InvalidAmount => "Conversion rate not parseable",
            ApiError::InvalidRatio => "Conversion rate was not valid",
        }
    }
}

#[async_trait]
pub trait Api {
    async fn get_rate(&self, from: &str, to: &str, date: &str) -> anyhow::Result<Decimal>;
}

#[derive(Clone)]
pub struct Currency {
    client: reqwest::Client,
    base_url: &'static str,
}

impl Currency {
    pub fn new() -> Self {
        Currency {
            client: reqwest::Client::new(),
            base_url: BASE_URL,
        }
    }
}

#[async_trait]
impl Api for Currency {
    async fn get_rate(&self, from: &str, to: &str, date: &str) -> anyhow::Result<Decimal> {
        let iso_from = from.to_lowercase();
        let iso_to = to.to_lowercase();
        let url = format!("{}/{}/currencies/{}/{}.json", self.base_url, date, iso_from, &iso_to);
        let client = self.client.clone();

        let resp = client.get(url)
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;
    
        let rate = resp.get(&iso_to).ok_or_else(|| ApiError::RateNotAvailable)?;
        let value = match rate {
            Value::Number(n) => n.as_f64().and_then(Decimal::from_f64),
            _ => anyhow::bail!(ApiError::InvalidAmount)
        };
        let rate = value.ok_or_else(|| ApiError::InvalidRatio)?;
    
        Ok(rate)
    }
}
