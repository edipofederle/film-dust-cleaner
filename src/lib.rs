use opencv::{
    core::{self, Mat, Point, Size},
    imgcodecs,
    imgproc,
    photo,
    prelude::*,
};

pub fn clean(input_path: &str, output_path: &str, sigma: f64, threshold: f64, inpaint_radius: f64) -> opencv::Result<()> {
    let img = imgcodecs::imread(input_path, imgcodecs::IMREAD_GRAYSCALE)?;
    if img.empty() {
        return Err(opencv::Error::new(opencv::core::StsError, format!("Could not read image: {}", input_path)));
    }

    let mut background = Mat::default();
    imgproc::gaussian_blur(
        &img,
        &mut background,
        Size::new(0, 0),
        sigma,
        sigma,
        core::BORDER_DEFAULT,
        core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    let mut diff = Mat::default();
    core::subtract(&img, &background, &mut diff, &core::no_array(), -1)?;

    let mut scratch_mask = Mat::default();
    imgproc::threshold(&diff, &mut scratch_mask, threshold, 255.0, imgproc::THRESH_BINARY)?;

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
    photo::inpaint(&img, &dilated_mask, &mut cleaned, inpaint_radius, photo::INPAINT_TELEA)?;

    imgcodecs::imwrite(output_path, &cleaned, &core::Vector::new())?;
    Ok(())
}
