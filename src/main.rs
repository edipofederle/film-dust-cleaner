use clap::Parser;
use opencv::{
    core::{self, Mat, Point, Size},
    imgcodecs,
    imgproc,
    photo,
    prelude::*,
};

#[derive(Parser)]
#[command(about = "Remove dust and scratches from scanned images.")]
struct Args {
    input: String,
    output: String,
    #[arg(long, default_value_t = 15.0)]
    sigma: f64,
    #[arg(long, default_value_t = 30.0)]
    threshold: f64,
    #[arg(long, default_value_t = 5.0)]
    inpaint_radius: f64,
}

fn main() -> opencv::Result<()> {
    let args = Args::parse();

    let img = imgcodecs::imread(&args.input, imgcodecs::IMREAD_GRAYSCALE)?;
    if img.empty() {
        eprintln!("Could not read image: {}", args.input);
        std::process::exit(1);
    }

    // Estimate local background
    let mut background = Mat::default();
    imgproc::gaussian_blur(
        &img,
        &mut background,
        Size::new(0, 0),
        args.sigma,
        args.sigma,
        core::BORDER_DEFAULT,
        core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    // Find pixels significantly brighter than their local neighbourhood
    let mut diff = Mat::default();
    core::subtract(&img, &background, &mut diff, &core::no_array(), -1)?;

    let mut scratch_mask = Mat::default();
    imgproc::threshold(&diff, &mut scratch_mask, args.threshold, 255.0, imgproc::THRESH_BINARY)?;

    // Expand mask slightly to cover scratch edges
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_RECT,
        Size::new(3, 3),
        Point::new(-1, -1),
    )?;
    let mut dilated_mask = Mat::default();
    imgproc::dilate(
        &scratch_mask,
        &mut dilated_mask,
        &kernel,
        Point::new(-1, -1),
        2,
        core::BORDER_CONSTANT,
        imgproc::morphology_default_border_value()?,
    )?;

    let mut cleaned = Mat::default();
    photo::inpaint(&img, &dilated_mask, &mut cleaned, args.inpaint_radius, photo::INPAINT_TELEA)?;

    imgcodecs::imwrite(&args.output, &cleaned, &core::Vector::new())?;
    println!("Saved to {}", args.output);

    Ok(())
}
