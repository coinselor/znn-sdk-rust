//! Node-optional API entry-point example.

use znn_sdk_rust::zenon::Zenon;

/// Runs the API example for `url` and returns the message that would be printed.
pub async fn run_with_url(url: &str) -> String {
    match Zenon::connect(url, false).await {
        Ok(zenon) => match zenon.ledger.get_frontier_momentum().await {
            Ok(momentum) => format!("frontier momentum height={}", momentum.height()),
            Err(error) => format!("No connection or request failed: {error}"),
        },
        Err(error) => format!("could not connect to {url}: {error}"),
    }
}

#[tokio::main]
#[allow(dead_code)]
async fn main() {
    let url = std::env::var("ZNN_NODE_URL").unwrap_or_else(|_| "ws://127.0.0.1:35998".to_string());
    println!("{}", run_with_url(&url).await);
}
