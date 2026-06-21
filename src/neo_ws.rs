use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;

use crate::{API_BASE_ADDRESS, ApiResponse, ClientInfo, Date, RequestError, common_errors};

/// Client for the Near Earth Object Web Service API
pub struct NeoWs {
    client_info: Arc<ClientInfo>,
}

impl NeoWs {
    pub(super) fn new(client_info: Arc<ClientInfo>) -> Self {
        Self { client_info }
    }

    /// Retreive a list of asteroids based on their closest approach date to Earth
    pub async fn feed(
        &self,
        start_date: Date,
        end_date: Date,
    ) -> Result<ApiResponse<NeoWsFeedResponse>, RequestError> {
        let query_params = [
            ("start_date", start_date.to_string()),
            ("end_date", end_date.to_string()),
            ("api_key", self.client_info.api_key.clone()),
        ];

        let response = self
            .client_info
            .client
            .get(format!("{API_BASE_ADDRESS}/neo/rest/v1/feed"))
            .query(&query_params)
            .send()
            .await?;

        common_errors(&response)?;

        let headers = response.headers().clone();
        let object = response.json::<NeoWsFeedResponse>().await?;

        let api_response = ApiResponse::new_with_headermap(object, &headers)?;

        Ok(api_response)
    }

    /// Lookup an asteroid based on its NASA JPL small body ID
    pub async fn lookup(&self, id: u32) -> Result<ApiResponse<NeoWsObject>, RequestError> {
        let query_params = [("api_key", self.client_info.api_key.clone())];

        let response = self
            .client_info
            .client
            .get(format!("{API_BASE_ADDRESS}/neo/rest/v1/neo/{id}"))
            .query(&query_params)
            .send()
            .await?;

        common_errors(&response)?;

        let headers = response.headers().clone();
        let object = response.json::<NeoWsObject>().await?;

        let api_response = ApiResponse::new_with_headermap(object, &headers)?;

        Ok(api_response)
    }

    /// Browse the overall Asteroid data set
    ///
    /// `size` is the amount of objects that should be returned, the API will return at maximum 20
    /// per page
    pub async fn browse(
        &self,
        page: u32,
        size: u32,
    ) -> Result<ApiResponse<NeoWsBrowseResponse>, RequestError> {
        let query_params = [
            ("page", page.to_string()),
            ("size", size.to_string()),
            ("api_key", self.client_info.api_key.clone()),
        ];

        let response = self
            .client_info
            .client
            .get(format!("{API_BASE_ADDRESS}/neo/rest/v1/neo/browse"))
            .query(&query_params)
            .send()
            .await?;

        common_errors(&response)?;

        let headers = response.headers().clone();
        let browse_response = response.json::<NeoWsBrowseResponse>().await?;

        let api_response = ApiResponse::new_with_headermap(browse_response, &headers)?;

        Ok(api_response)
    }
}

/// Response of the [`NeoWs::feed`] endpoint
#[derive(Debug, Deserialize)]
pub struct NeoWsFeedResponse {
    /// Amount of objects in [`near_earth_objects`]
    pub element_count: u32,
    /// Objects by date
    pub near_earth_objects: HashMap<Date, Vec<NeoWsObject>>,
}

/// Represents an object from the NeoWs API
#[derive(Debug, Deserialize)]
pub struct NeoWsObject {
    /// Object Id
    pub id: String,
    /// NeoWs reference id
    pub neo_reference_id: String,
    /// Name of the object
    pub name: String,
    /// URL to a lookup of this body on https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html
    pub nasa_jpl_url: String,
    /// The absolute magnitude
    pub absolute_magnitude_h: f32,
    /// Whether or not this is a potentially hazardous asteroid
    pub is_potentially_hazardous_asteroid: bool,
    /// A list of close approaches
    pub close_approach_data: Vec<NeoWsCloseApproach>,
    /// Orbital Info of the object, not present in [`NeoWs::feed`] requests
    pub orbital_data: Option<NeoWsOrbitalInfo>,
    /// Whether or not this object is a sentry object
    pub is_sentry_object: bool,
}

/// Contains the estimated diameter of an object in multiple units
#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct NeoWsEstimatedDiameter {
    pub kilometers: NeoWsEstimatedDiameterSingle,
    pub meters: NeoWsEstimatedDiameterSingle,
    pub miles: NeoWsEstimatedDiameterSingle,
    pub feet: NeoWsEstimatedDiameterSingle,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct NeoWsEstimatedDiameterSingle {
    pub estimated_diameter_min: f64,
    pub estimated_diameter_max: f64,
}

/// Contains data about a close approach
#[derive(Debug, Deserialize)]
pub struct NeoWsCloseApproach {
    close_approach_date: Date,
    close_approach_date_full: String,
    epoch_date_close_approach: i64,
    /// Body the object is orbiting at the time of the approach
    pub orbiting_body: String,
    /// Relative velocity
    pub relative_velocity: NeoWsVelocity,
    /// Miss distance
    pub miss_distance: NeoWsMissDistance,
}

impl NeoWsCloseApproach {
    /// Returns the close approach time as a epoch timestamp
    pub fn timestamp(&self) -> i64 {
        self.epoch_date_close_approach
    }
}

/// Contains the velocity in various units
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct NeoWsVelocity {
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub kilometers_per_second: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub kilometers_per_hour: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub miles_per_hour: f64,
}

/// Contains distance in various units
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct NeoWsMissDistance {
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub astronomical: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub lunar: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub kilometers: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub miles: f64,
}

/// Orbital info of an object
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub struct NeoWsOrbitalInfo {
    pub orbit_id: String,
    pub orbit_determination_date: String,
    pub first_observation_date: Date,
    pub last_observation_date: Date,
    pub data_arc_in_days: u32,
    pub observations_used: u32,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub orbit_uncertainty: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub minimum_orbit_intersection: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub jupiter_tisserand_invariant: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub epoch_osculation: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub eccentricity: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub semi_major_axis: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub inclination: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub ascending_node_longitude: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub orbital_period: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub perihelion_distance: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub perihelion_argument: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub aphelion_distance: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub perihelion_time: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub mean_anomaly: f64,
    #[serde(deserialize_with = "crate::string_as_f64")]
    pub mean_motion: f64,
    pub equinox: String,
}

/// Response from [`NeoWs::browse`]
#[derive(Debug, Deserialize)]
pub struct NeoWsBrowseResponse {
    /// Information about the amount of objects in the API
    pub page: NeoWsBrowsePageInfo,
    /// Objects returned by the API
    pub near_earth_objects: Vec<NeoWsObject>,
}

/// Information about the amount of objects in the API
#[derive(Debug, Deserialize)]
pub struct NeoWsBrowsePageInfo {
    /// Amount of objects returned
    pub size: u32,
    /// Total objects in the API
    pub total_elements: u32,
    /// Total amount of pages at this page size
    pub total_pages: u32,
}
