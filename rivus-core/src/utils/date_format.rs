use chrono::NaiveDateTime;
use serde::{self, Serializer};

pub fn serialize_with_custom_format<S>(
    date: &Option<NaiveDateTime>,
    format: &str,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(dt) => serializer.serialize_str(&dt.format(format).to_string()),
        None => serializer.serialize_none(),
    }
}

macro_rules! define_format {
    ($name:ident, $format:expr) => {
        pub mod $name {
            use super::*;
            pub fn serialize<S>(
                date: &Option<NaiveDateTime>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serialize_with_custom_format(date, $format, serializer)
            }
        }
    };
}

// 预定义一些常用格式
define_format!(standard, "%Y-%m-%d %H:%M:%S");
define_format!(date_only, "%Y-%m-%d");
