use serde::Deserialize;

use crate::Date;

/// Represents a query to the APOD API
pub enum ApodQuery {
    /// Query image from a single date
    Single(Date),
    /// Query images from a range of dates
    Range {
        /// The start of the range
        start_date: Date,
        /// The end of the range
        end_date: Date,
    },
    /// Return x random images
    /// Maximum of 100 images
    Count(u8),
    /// Get the current APOD
    Today,
}

/// A response from the APOD API
#[derive(Deserialize)]
pub struct ApodResponse {
    /// The title of the image
    pub title: String,
    /// Date of image
    pub date: Date,
    /// The supplied text explanation of the image
    pub explanation: String,
    /// The URL of the APOD image or video of the day
    pub url: String,
    /// The URL for any high-resolution image that day
    pub hdurl: Option<String>,
    /// The name of the copyright holder
    pub copyright: Option<String>,
}
