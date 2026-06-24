use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::fs;

mod config;
use config::Config;

#[derive(Parser)]
#[command(about = "Remove dust and scratches from scanned film photos.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Remove dust, scratches, and optionally grain from a scan
    Clean {
        input: String,
        /// Output path. If omitted, output_dir from config is used.
        output: Option<String>,
        #[arg(long, default_value_t = 15.0)]
        sigma: f64,
        #[arg(long, default_value_t = 30.0)]
        threshold: f64,
        #[arg(long, default_value_t = 5.0)]
        inpaint_radius: f64,
        /// Grain reduction strength (0 = disabled, 3–15 typical range)
        #[arg(long, default_value_t = 0.0)]
        denoise: f32,
        /// Treat input as a colour negative and invert before cleaning
        #[arg(long, default_value_t = false)]
        invert: bool,
        /// Exposure adjustment in EV stops (-2.0 to +2.0, 0 = no change)
        #[arg(long, default_value_t = 0.0)]
        exposure: f64,
        /// Contrast multiplier (1.0 = no change, >1 increases contrast)
        #[arg(long, default_value_t = 1.0)]
        contrast: f64,
    },
    /// Invert a colour negative scan to a positive
    Invert {
        input: String,
        output: String,
    },
    /// Start the web UI server
    Serve {
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = Config::load();

    match cli.command {
        Command::Clean { input, output, sigma, threshold, inpaint_radius, denoise, invert, exposure, contrast } => {
            let output = match config.resolve_output(&input, output) {
                Ok(p) => p,
                Err(e) => { eprintln!("Error: {e}"); std::process::exit(1); }
            };
            if let Err(e) = film_dust_cleaner::clean(&input, &output, sigma, threshold, inpaint_radius, denoise, invert, exposure, contrast) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Saved to {output}");
        }
        Command::Invert { input, output } => {
            if let Err(e) = film_dust_cleaner::invert_negative(&input, &output) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Saved to {output}");
        }
        Command::Serve { port } => {
            let app = Router::new()
                .route("/", get(index))
                .route("/clean", post(clean_handler))
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

            let addr = format!("0.0.0.0:{port}");
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            println!("Listening on http://localhost:{port}");
            axum::serve(listener, app).await.unwrap();
        }
    }
}

async fn index() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

async fn clean_handler(mut multipart: Multipart) -> Result<Response, StatusCode> {
    let mut image_data: Option<Vec<u8>> = None;
    let mut sigma = 15.0f64;
    let mut threshold = 30.0f64;
    let mut inpaint_radius = 5.0f64;
    let mut denoise = 0.0f32;
    let mut invert = false;
    let mut exposure = 0.0f64;
    let mut contrast = 1.0f64;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        match field.name().unwrap_or("") {
            "image" => {
                image_data = Some(field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec());
            }
            "sigma" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { sigma = v; }
            }
            "threshold" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { threshold = v; }
            }
            "inpaint_radius" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { inpaint_radius = v; }
            }
            "denoise" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { denoise = v; }
            }
            "invert" => {
                invert = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)? == "true";
            }
            "exposure" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { exposure = v; }
            }
            "contrast" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() { contrast = v; }
            }
            _ => {}
        }
    }

    let image_data = image_data.ok_or(StatusCode::BAD_REQUEST)?;

    let result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let input = tempfile::Builder::new().suffix(".jpg").tempfile()?;
        let output = tempfile::Builder::new().suffix(".jpg").tempfile()?;

        fs::write(input.path(), &image_data)?;
        film_dust_cleaner::clean(
            input.path().to_str().unwrap(),
            output.path().to_str().unwrap(),
            sigma, threshold, inpaint_radius, denoise, invert, exposure, contrast,
        )?;

        Ok(fs::read(output.path())?)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(([(header::CONTENT_TYPE, "image/jpeg")], result).into_response())
}
