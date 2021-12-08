use anyhow::{bail, Result};
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rusty_money::{
    iso::{self, Currency},
    ExchangeRate, Money, MoneyError,
};
use sqlx::types::BigDecimal;

pub(crate) trait ToBigDecimal {
    fn to_bigdecimal(&self) -> BigDecimal;
}

impl ToBigDecimal for f64 {
    fn to_bigdecimal(&self) -> BigDecimal {
        BigDecimal::from_f64(*self).unwrap_or_default()
    }
}

pub(crate) fn to_bigdecimal<F64: ToBigDecimal>(f: F64) -> BigDecimal {
    F64::to_bigdecimal(&f)
}

pub(crate) trait RoundTwo {
    fn round_two(&self) -> f64;
}

impl RoundTwo for BigDecimal {
    fn round_two(&self) -> f64 {
        let to = self.to_f64().unwrap_or_default();
        (to * 100.0).round() / 100.0
    }
}

pub(crate) fn round_two<R: RoundTwo>(value: &R) -> f64 {
    R::round_two(value)
}

trait ConversionRate {
    fn get_rate_for(&self, foreign: &Currency) -> Result<Decimal>;
}

// source: https://mercati.ilsole24ore.com/tassi-e-valute/valute/contro-euro/cambio/EURUS.FX
impl ConversionRate for Currency {
    fn get_rate_for(&self, foreign: &Currency) -> Result<Decimal> {
        match (self.iso_alpha_code, foreign.iso_alpha_code) {
            ("EUR", "EUR") => Ok(dec!(1)),
            ("USD", "USD") => Ok(dec!(1)),
            ("EUR", "USD") => Ok(dec!(1.131857)),
            ("USD", "EUR") => Ok(dec!(0.86207)),
            _ => bail!("Currency not supported yet! :( Feel free to open a PR :)!"),
        }
    }
}

pub(crate) fn exchange(from: &str, to: &str, amount: f64, rate: Decimal) -> Result<f64> {
    let iso_from = iso::find(from).ok_or(MoneyError::InvalidCurrency)?;
    let iso_to = iso::find(to).ok_or(MoneyError::InvalidCurrency)?;
    let amount = Decimal::from_f64(amount).ok_or(MoneyError::InvalidAmount)?;

    let amount = ExchangeRate::new(iso_from, iso_to, rate)?
        .convert(Money::from_decimal(amount, iso_from))
        .map(|money| *money.amount())?;

    amount
        .to_f64()
        .ok_or_else(|| anyhow::anyhow!("Conversion failed for {}", amount))
}

pub(crate) fn get_datetime_zero() -> DateTime<Utc> {
    let zero = chrono::NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0);
    DateTime::<Utc>::from_utc(zero, Utc)
}

#[cfg(test)]
mod tests {
    use crate::util::{get_datetime_zero, round_two, to_bigdecimal};
    use sqlx::types::BigDecimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    use super::exchange;

    #[test]
    fn test_exchange_usd_eur_works() {
        let amount = 10;
        let eur = exchange("USD", "EUR", amount.into(), dec!(4.20)).unwrap();

        assert_eq!(eur, 42 as f64);
    }

    #[test]
    fn test_exchange_eur_usd_works() {
        let amount = 10;
        let usd = exchange("EUR", "USD", amount.into(), dec!(42)).unwrap();
        assert_eq!(usd, 420.0 as f64);
    }

    #[test]
    fn test_exchange_eur_usd2_works() {
        let amount = 1;
        let usd = exchange("EUR", "USD", amount.into(), dec!(0.42)).unwrap();
        assert_eq!(usd, 0.42 as f64);
    }

    // #[test]
    // fn test_exchange_others_not_works() {
    //     let amount = 10;
    //     let change = exchange("EUR", "CAD", amount.into(), dec!(0.22));
    //     assert!(change.is_err());
    // }

    #[test]
    fn test_datetime_zero_works() {
        let expect = get_datetime_zero();
        assert_eq!(expect.to_string(), String::from("1970-01-01 00:00:00 UTC"));
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
        assert_eq!(round_two(&BigDecimal::from_str("123.12").unwrap()), 123.12);
    }
}
