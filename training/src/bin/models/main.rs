mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run().await
}
