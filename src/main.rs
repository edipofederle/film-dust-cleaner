use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(about = "Remove dust and scratches from scanned film photos.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Clean {
        input: String,
        output: String,
        #[arg(long, default_value_t = 15.0)]
        sigma: f64,
        #[arg(long, default_value_t = 30.0)]
        threshold: f64,
        #[arg(long, default_value_t = 5.0)]
        inpaint_radius: f64,
    },
    Serve {
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Clean { input, output, sigma, threshold, inpaint_radius } => {
            if let Err(e) = film_dust_cleaner::clean(&input, &output, sigma, threshold, inpaint_radius) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            println!("Saved to {output}");
        }
        Command::Serve { port } => {
            let app = Router::new()
                .route("/", get(index))
                .route("/clean", post(clean_handler))
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)); // 50 MB

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

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        match field.name().unwrap_or("") {
            "image" => {
                image_data = Some(field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec());
            }
            "sigma" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() {
                    sigma = v;
                }
            }
            "threshold" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() {
                    threshold = v;
                }
            }
            "inpaint_radius" => {
                if let Ok(v) = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?.parse() {
                    inpaint_radius = v;
                }
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
            sigma,
            threshold,
            inpaint_radius,
        )?;

        Ok(fs::read(output.path())?)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(([(header::CONTENT_TYPE, "image/jpeg")], result).into_response())
}
