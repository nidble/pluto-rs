use anyhow::{bail, Result};
use rust_decimal::Decimal;
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
            _ => bail!("Currency not supported yet! :(, please add PR to support it!")
        }
    }
}

pub fn exchange(from: String, to: String, amount: i64) -> Result<Decimal> {
    let iso_from = iso::find(&from).ok_or(MoneyError::InvalidCurrency)?;
    let iso_to = iso::find(&to).ok_or(MoneyError::InvalidCurrency)?;
    
    let rate = iso_from.get_rate_for(iso_to)?;
    let ok = ExchangeRate::new(iso_from, iso_to, rate)?
        .convert(Money::from_minor(amount * 100, iso_from))
        .map(|money| money.amount().clone())?;
    
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::exchange;

    #[test]
    fn test_exchange_usd_eur_works() {
        let amount = 10;
        let eur = exchange("USD".to_string(), "EUR".to_string(), amount.into()).unwrap();

        assert_eq!(eur, dec!(8.6207));
    }

    #[test]
    fn test_exchange_eur_usd_works() {
        let amount = 10;
        let usd = exchange("EUR".to_string(), "USD".to_string(), amount.into()).unwrap();
        assert_eq!(usd, dec!(11.31857));
    }

    #[test]
    fn test_exchange_others_not_works() {
        let amount = 10;
        let change = exchange("EUR".to_string(), "CAD".to_string(), amount.into());
        assert!(change.is_err());
    }
}