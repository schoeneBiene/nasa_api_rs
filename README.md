# `nasa_api_rs`
A wrapper of the NASA API for Rust

## Example
```rust
use nasa_api::{Client, RequestError, apod::ApodQuery};

#[tokio::main]
async fn main() -> Result<(), RequestError> {
    // This uses DEMO_KEY
    let client = Client::default();

    // Query today's Astronomy Picture of the Day
    let apod_response = client.apod(ApodQuery::Today).await?;

    println!(
        "Title of today's Astronomy Picture of the Day: {}",
        apod_response.title
    );

    Ok(())
}
```
