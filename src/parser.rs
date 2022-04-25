use crate::transaction::Transaction;
use csv::ReaderBuilder;
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer};

pub struct Parser {}

impl Parser {
    pub fn parse(file_path: &str) -> Result<Vec<Transaction>, csv::Error> {
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(file_path)?;

        let mut result = Vec::new();
        for r in rdr.deserialize() {
            match r {
                Err(..) => continue,
                Ok(tx) => result.push(tx),
            }
        }
        Ok(result)
    }
}

pub fn arbitrary_tx_amount<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Amount<T> {
        Number(T),
        String(String),
    }

    match Amount::<T>::deserialize(deserializer)? {
        Amount::String(s) if s.is_empty() => Ok(T::default()),
        Amount::Number(i) => Ok(i),
        Amount::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
    }
}
