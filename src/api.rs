use std::{collections::HashMap, fmt, error};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::Value;

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

// https://raw.githubusercontent.com/fawazahmed0/currency-api/1/latest/currencies/usd/eur.json { "date": "2021-12-07", "eur": 0.88645 }
pub(crate) async fn get_rate(from: &str, to: &str, date: impl Into<String>) -> anyhow::Result<Decimal> {
    let iso_from = from.to_lowercase();
    let iso_to = to.to_lowercase();
    let url = format!("{}/{}/currencies/{}/{}.json", BASE_URL, date.into(), iso_from, &iso_to);
    let resp = reqwest::get(url)
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
