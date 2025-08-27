use clap::Parser;
use secrecy::SecretBox;

#[derive(Debug, Parser)]
/// Configuration used to start the server
struct Config {
    /// Url used to connect to the database instance
    #[clap(long, env = "DATABASE_URL", hide_env_values = true)]
    database_url: SecretBox<str>,
    /// Host to bind to
    #[clap(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,
    /// Port to bind to
    #[clap(long, env = "PORT", default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    gecko_recipes::server(gecko_recipes::Config {
        database_url: config.database_url,
        host: config.host,
        port: config.port,
    })
    .await?;
    Ok(())
}
