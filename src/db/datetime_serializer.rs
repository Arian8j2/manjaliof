use chrono::{DateTime, Utc, TimeZone};
use serde::{self, Deserialize, Serializer, Deserializer};

const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let date_string = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&date_string)
}

pub fn deserialize<'a, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error> where D: Deserializer<'a> {
    let date_string = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&date_string, FORMAT).map_err(serde::de::Error::custom)
}
