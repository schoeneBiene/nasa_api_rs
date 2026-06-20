//! A wrapper for the NASA API

#![warn(missing_docs)]

use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::Deref,
};

#[cfg(feature = "apod")]
use crate::apod::{ApodQuery, ApodResponse};
use reqwest::{
    Response, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::{
    Deserialize, Serialize,
    de::{Unexpected, Visitor},
};

const API_BASE_ADDRESS: &str = "https://api.nasa.gov";

/// The Astronomy Picture of the Day API
#[cfg(feature = "apod")]
pub mod apod;

/// Represents a response from the API
///
/// This struct implements [`Deref`], so you can directly access the inner `T`:
///
/// ```rust
/// struct MyData {
///     value: i32,
/// }
///
/// impl MyData {
///     fn get_value(&self) -> i32 {
///         self.value
///     }
/// }
///
/// let data = MyData { value: 5 };
/// let api_response = ApiResponse::new(data, 100, 99);
///
/// let value = api_response.get_value();
/// ```
#[derive(Debug, Clone)]
pub struct ApiResponse<T> {
    response: T,
    ratelimit: u32,
    ratelimit_remaining: u32,
}

impl<T> ApiResponse<T> {
    #[doc(hidden)]
    pub fn new(response: T, ratelimit: u32, ratelimit_remaining: u32) -> Self {
        Self {
            response,
            ratelimit,
            ratelimit_remaining,
        }
    }

    #[doc(hidden)]
    pub fn new_with_headermap(
        inner: T,
        headers: &HeaderMap<HeaderValue>,
    ) -> Result<Self, RequestError> {
        let ratelimit_str = headers
            .get("X-RateLimit-Limit")
            .ok_or(RequestError::UnexpectedError(
                "couldn't get ratelimit header",
            ))?
            .to_str()
            .map_err(|_| {
                RequestError::UnexpectedError("couldn't convert HeaderValue into string")
            })?;

        let ratelimit_remaining_str = headers
            .get("X-RateLimit-Remaining")
            .ok_or(RequestError::UnexpectedError(
                "couldn't get ratelimit remaining header",
            ))?
            .to_str()
            .map_err(|_| {
                RequestError::UnexpectedError("couldn't convert HeaderValue into string")
            })?;

        let ratelimit = ratelimit_str
            .parse()
            .map_err(|_| RequestError::UnexpectedError("couldn't parse ratelimit as number"))?;
        let ratelimit_remaining = ratelimit_remaining_str
            .parse()
            .map_err(|_| RequestError::UnexpectedError("couldn't parse ratelimit as number"))?;

        Ok(Self::new(inner, ratelimit, ratelimit_remaining))
    }

    /// Gets the default hourly ratelimit
    pub fn ratelimit(&self) -> u32 {
        self.ratelimit
    }

    /// Gets the remaining amount of requests
    /// This is reset hourly
    pub fn ratelimit_remaining(&self) -> u32 {
        self.ratelimit_remaining
    }
}

impl<T> Deref for ApiResponse<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("ratelimit reached")]
    Ratelimit,

    #[error("parsing returned JSON failed: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("unexpected error: {0}")]
    UnexpectedError(&'static str),
}

#[cfg(test)]
#[test]
fn api_response_test() {
    struct A {
        b: i32,
    }

    impl A {
        pub fn new() -> Self {
            Self { b: 5 }
        }

        fn get_b(&self) -> i32 {
            self.b
        }
    }

    let a = A::new();
    let api_response = ApiResponse::new(a, 100, 99);

    assert_eq!(api_response.get_b(), 5);
    assert_eq!(api_response.ratelimit(), 100);
    assert_eq!(api_response.ratelimit_remaining(), 99);
}

/// Returns a [`Response`] with a [`RequestError`], which can be used with the question mark
/// operator to filter out bad responses
pub(crate) fn common_errors(response: &Response) -> Result<(), RequestError> {
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(RequestError::Ratelimit);
    }

    Ok(())
}

/// The API client
pub struct Client {
    api_key: String,
    client: reqwest::Client,
}

impl Client {
    /// Create a new [`Client`] with the given API key
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Get the Astronomy Picture of the Day using the specified query
    #[cfg(feature = "apod")]
    pub async fn apod(&self, query: ApodQuery) -> Result<ApiResponse<ApodResponse>, RequestError> {
        let mut query_params = HashMap::new();

        match query {
            ApodQuery::Single(date) => {
                query_params.insert("date", date.to_string());
            }
            ApodQuery::Range {
                start_date,
                end_date,
            } => {
                query_params.insert("start_date", start_date.to_string());
                query_params.insert("end_date", end_date.to_string());
            }
            ApodQuery::Count(count) => {
                query_params.insert("count", count.to_string());
            }
            ApodQuery::Today => {}
        }

        query_params.insert("api_key", self.api_key.clone());

        let response = self
            .client
            .get(format!("{API_BASE_ADDRESS}/planetary/apod"))
            .query(&query_params)
            .send()
            .await?;

        common_errors(&response)?;

        let headermap = response.headers().clone();
        let apod_response = response.json::<ApodResponse>().await?;

        let api_response = ApiResponse::new_with_headermap(apod_response, &headermap)?;

        Ok(api_response)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new("DEMO_KEY".to_string())
    }
}

/// Represents a date
///
/// Implements [`serde::Serialize`] and [`serde::Deserialize`][^note] for YYYY-MM-DD format
///
/// [^note]: Deserialize implementation does not strictly check for the YYYY-MM-DD format
pub struct Date {
    /// The year
    pub year: u32,
    /// The month
    pub month: u8,
    /// The day
    pub day: u8,
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateVisitor)
    }
}

impl Display for Date {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str(&format!(
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        ))?;

        Ok(())
    }
}

struct DateVisitor;

impl<'de> Visitor<'de> for DateVisitor {
    type Value = Date;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut parts = v.split('-');

        // TODO: maybe dry this (get it?)
        let year_str = parts
            .next()
            .ok_or(serde::de::Error::invalid_value(Unexpected::Str(v), &self))?;
        let month_str = parts
            .next()
            .ok_or(serde::de::Error::invalid_value(Unexpected::Str(v), &self))?;
        let day_str = parts
            .next()
            .ok_or(serde::de::Error::invalid_value(Unexpected::Str(v), &self))?;

        let year = year_str.parse();
        let month = month_str.parse();
        let day = day_str.parse();

        if year.is_err() || month.is_err() || day.is_err() {
            return Err(serde::de::Error::invalid_value(Unexpected::Str(v), &self));
        }

        Ok(Date {
            year: year.unwrap(),
            month: month.unwrap(),
            day: day.unwrap(),
        })
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string in the format of YYYY-MM-DD")?;

        Ok(())
    }
}
