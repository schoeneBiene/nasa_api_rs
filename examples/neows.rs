use nasa_api::{Client, Date, RequestError};

#[tokio::main]
async fn main() -> Result<(), RequestError> {
    // This uses DEMO_KEY
    let client = Client::new("5Xa1WjF5TRxnrajkt6EW5ybz1Hc540vabqBCLfDd".to_string());

    let neo_ws = client.neo_ws();
    let response = neo_ws
        .feed(
            Date {
                year: 2015,
                month: 9,
                day: 7,
            },
            Date {
                year: 2015,
                month: 9,
                day: 8,
            },
        )
        .await?;

    let first = response
        .near_earth_objects
        .get(&Date {
            year: 2015,
            month: 9,
            day: 7,
        })
        .unwrap()
        .first()
        .expect("no objects were returned");

    let again = neo_ws
        .lookup(first.neo_reference_id.parse().unwrap())
        .await?;

    assert_eq!(first.absolute_magnitude_h, again.absolute_magnitude_h);

    println!("{:#?}", again);

    let browse_response = neo_ws.browse(10, 20).await?;

    println!("Total pages: {}", browse_response.page.total_pages);

    Ok(())
}
