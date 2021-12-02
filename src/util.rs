use anyhow::{bail, Result};
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rusty_money::{ExchangeRate, Money, MoneyError, iso::{self, Currency}};
use rust_decimal_macros::dec;

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
            _ => bail!("Currency not supported yet! :( Feel free to open a PR :)!")
        }
    }
}

pub fn exchange(from: &str, to: &str, amount: f64) -> Result<f64> {
    let iso_from = iso::find(from).ok_or(MoneyError::InvalidCurrency)?;
    let iso_to = iso::find(to).ok_or(MoneyError::InvalidCurrency)?;
    let amount = Decimal::from_f64(amount).ok_or(MoneyError::InvalidAmount)?;

    let rate = iso_from.get_rate_for(iso_to)?;
    let amount = ExchangeRate::new(iso_from, iso_to, rate)?
        .convert(Money::from_decimal(amount, iso_from))
        .map(|money| *money.amount())?;
    
    amount.to_f64().ok_or_else(|| anyhow::anyhow!("Conversion failed for {}", amount))
}

pub fn get_datetime_zero() -> DateTime<Utc> {
    let zero = chrono::NaiveDate::from_ymd(1970, 1, 1)
        .and_hms_milli(0, 0, 0, 0);
    DateTime::<Utc>::from_utc(zero, Utc)
}

#[cfg(test)]
mod tests {
    use crate::util::get_datetime_zero;

    use super::exchange;

    #[test]
    fn test_exchange_usd_eur_works() {
        let amount = 10;
        let eur = exchange("USD", "EUR", amount.into()).unwrap();

        assert_eq!(eur, 8.6207 as f64);
    }

    #[test]
    fn test_exchange_eur_usd_works() {
        let amount = 10;
        let usd = exchange("EUR", "USD", amount.into()).unwrap();
        assert_eq!(usd, 11.31857 as f64);
    }


    #[test]
    fn test_exchange_eur_usd2_works() {
        let amount = 1;
        let usd = exchange("EUR", "USD", amount.into()).unwrap();
        assert_eq!(usd, 1.131857 as f64);
    }

    #[test]
    fn test_exchange_others_not_works() {
        let amount = 10;
        let change = exchange("EUR", "CAD", amount.into());
        assert!(change.is_err());
    }


    #[test]
    fn test_datetime_zero_works() {
        let expect = get_datetime_zero();
        assert_eq!(expect.to_string(), String::from("1970-01-01 00:00:00 UTC"));
    }
}