use std::fmt;

use serde::{Deserialize, Serialize};

use self::account::Balance;

pub mod account;
pub mod candlestick;
pub mod channel;
pub mod event;
pub mod fee;
pub mod order;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeSide {
    Buy,
    Sell,
}

impl fmt::Display for TradeSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TradeSide::Buy => write!(f, "BUY"),
            TradeSide::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bid,
    Offer,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    Cancelled,
    Expired,
    Failed,
}

pub(crate) mod string_or_float {
    use std::fmt;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => {
                if s == "INF" {
                    Ok(f64::INFINITY)
                } else {
                    s.parse().map_err(de::Error::custom)
                }
            }
            StringOrFloat::Float(i) => Ok(i),
        }
    }
}

pub(crate) mod string_or_i64 {
    use serde::{de, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrI64 {
            String(String),
            Int(i64),
        }

        match StringOrI64::deserialize(deserializer)? {
            StringOrI64::String(s) => s.parse().map_err(de::Error::custom),
            StringOrI64::Int(i) => Ok(i),
        }
    }
}

pub(crate) mod string_or_float_opt {
    use std::fmt;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        match value {
            Some(v) => crate::model::string_or_float::serialize(v, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        Ok(Some(crate::model::string_or_float::deserialize(
            deserializer,
        )?))
    }
}
