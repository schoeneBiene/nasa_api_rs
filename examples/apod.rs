use nasa_api::{Client, RequestError, apod::ApodQuery};

#[tokio::main]
async fn main() -> Result<(), RequestError> {
    // This uses DEMO_KEY
    let client = Client::default();

    let apod_response = client.apod(ApodQuery::Today).await?;

    println!(
        "Title of today's Astronomy Picture of the Day: {}",
        apod_response.title
    );

    Ok(())
}
