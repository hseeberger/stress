#[tokio::main]
async fn main() {
    // Error logging already happens in `stress::main`.
    let _ = stress::main().await;
}
