use std::sync::Arc;

use ajen_engine::context::{EngineConfig, EngineContext};
use axum::Router;
use clap::Parser;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod auth;
mod routes;
mod state;
mod tunnel;
mod ws;

use state::AppState;

#[derive(Parser)]
#[command(name = "ajen", about = "Ajen CLI — AI Employee Platform")]
struct Cli {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(long, env = "WORKSPACE_DIR", default_value = "./workspaces")]
    workspace_dir: String,

    #[arg(long, env = "MANIFESTS_DIR")]
    manifests_dir: Option<String>,

    /// Disable the Cloudflare Quick Tunnel
    #[arg(long, default_value = "false")]
    no_tunnel: bool,

    /// Don't open browser automatically
    #[arg(long, default_value = "false")]
    no_open: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let secret = auth::generate_secret();

    // Create engine context
    let engine = EngineContext::create(EngineConfig {
        database_url: std::env::var("DATABASE_URL").ok(),
        workspace_dir: cli.workspace_dir,
        manifests_dir: cli.manifests_dir,
        anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
        openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        gemini_api_key: std::env::var("GEMINI_API_KEY").ok(),
        ollama_base_url: std::env::var("OLLAMA_BASE_URL").ok(),
        ollama_enabled: std::env::var("OLLAMA_ENABLED")
            .ok()
            .map(|v| v == "true" || v == "1"),
        budget_company_limit_cents: None,
        budget_per_employee_limit_cents: None,
    })
    .await?;

    // Start tunnel and await URL (unless disabled)
    // Ensure cloudflared is installed (may auto-install), then start tunnel
    let tunnel_url = if !cli.no_tunnel {
        if let Some(rx) = tunnel::spawn_tunnel(cli.port).await {
            match tokio::time::timeout(std::time::Duration::from_secs(15), rx).await {
                Ok(Ok(url)) => {
                    // Give Cloudflare time to propagate the tunnel
                    eprint!("  Waiting for tunnel to propagate");
                    for _ in 0..10 {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        eprint!(".");
                    }
                    eprintln!(" ready!");
                    Some(url)
                }
                _ => {
                    eprintln!(
                        "  Warning: Tunnel failed to start within 15s, continuing without it."
                    );
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let local_url = format!("http://localhost:{}", cli.port);
    let version = env!("CARGO_PKG_VERSION");

    // Build connect link
    let connect_url = tunnel_url.as_ref().map(|turl| {
        let encoded_url = urlencoding::encode(turl);
        let encoded_secret = urlencoding::encode(&secret);
        format!("https://www.ajen.dev/cli_auth?url={encoded_url}&secret={encoded_secret}")
    });

    // Print startup box
    print_startup_box(
        version,
        &secret,
        &local_url,
        tunnel_url.as_deref(),
        connect_url.as_deref(),
    );

    // Open browser to connect link
    if !cli.no_open {
        if let Some(ref url) = connect_url {
            if let Err(e) = open::that(url) {
                eprintln!("  Could not open browser: {e}");
            }
        }
    }

    let state = AppState {
        engine: Arc::new(engine),
        secret: secret.clone(),
    };

    // CORS: allow all origins
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .merge(routes::health_router())
        .nest("/api", routes::api_router(state.secret.clone()))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

fn print_startup_box(
    version: &str,
    secret: &str,
    local_url: &str,
    tunnel_url: Option<&str>,
    connect_url: Option<&str>,
) {
    eprintln!();
    eprintln!("  ┌─────────────────────────────────────────────────┐");
    eprintln!("  │  Ajen CLI v{:<37}│", version);
    eprintln!("  │                                                 │");
    eprintln!("  │  Secret:  {:<40}│", secret);
    eprintln!("  │  Local:   {:<40}│", local_url);
    if let Some(turl) = tunnel_url {
        eprintln!("  │  Tunnel:  {:<40}│", turl);
    }
    eprintln!("  │                                                 │");
    if let Some(curl) = connect_url {
        eprintln!("  │  Connect: {:<40}│", curl);
        eprintln!("  │                                                 │");
    }
    eprintln!("  │  Ready. Waiting for commands.                   │");
    eprintln!("  └─────────────────────────────────────────────────┘");
    eprintln!();
}
