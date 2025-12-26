use chrono::{NaiveDate, NaiveDateTime};
use serde::{self, Serializer};

/// Trait for types that can be formatted with a date string.
pub trait DateFormattable {
    fn format_date(&self, fmt: &str) -> String;
    fn is_none(&self) -> bool;
}

impl DateFormattable for Option<NaiveDateTime> {
    fn format_date(&self, fmt: &str) -> String {
        match self {
            Some(dt) => dt.format(fmt).to_string(),
            None => String::new(),
        }
    }
    fn is_none(&self) -> bool {
        self.is_none()
    }
}

impl DateFormattable for Option<NaiveDate> {
    fn format_date(&self, fmt: &str) -> String {
        match self {
            Some(d) => d.format(fmt).to_string(),
            None => String::new(),
        }
    }
    fn is_none(&self) -> bool {
        self.is_none()
    }
}

pub fn serialize_with_custom_format<S, T>(
    date: &T,
    format: &str,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: DateFormattable,
{
    if date.is_none() {
        serializer.serialize_none()
    } else {
        serializer.serialize_str(&date.format_date(format))
    }
}

macro_rules! define_format {
    ($name:ident, $format:expr) => {
        pub mod $name {
            use super::*;
            pub fn serialize<S, T>(date: &T, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
                T: DateFormattable,
            {
                serialize_with_custom_format(date, $format, serializer)
            }
        }
    };
}

// 预定义一些常用格式
define_format!(standard, "%Y-%m-%d %H:%M:%S");
define_format!(date_only, "%Y-%m-%d");
