use opencv::{
    core::{self, Mat, Point, Size},
    imgcodecs,
    imgproc,
    photo,
    prelude::*,
};

pub fn clean(
    input_path: &str,
    output_path: &str,
    sigma: f64,
    threshold: f64,
    inpaint_radius: f64,
    denoise_strength: f32,
    invert: bool,
    exposure: f64,
    contrast: f64,
) -> opencv::Result<()> {
    let img = if invert {
        let color = imgcodecs::imread(input_path, imgcodecs::IMREAD_COLOR)?;
        if color.empty() {
            return Err(opencv::Error::new(core::StsError, format!("Could not read image: {}", input_path)));
        }
        let inverted = invert_mat(&color)?;
        let mut gray = Mat::default();
        imgproc::cvt_color(&inverted, &mut gray, imgproc::COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;
        gray
    } else {
        let img = imgcodecs::imread(input_path, imgcodecs::IMREAD_GRAYSCALE)?;
        if img.empty() {
            return Err(opencv::Error::new(core::StsError, format!("Could not read image: {}", input_path)));
        }
        img
    };

    let mut background = Mat::default();
    imgproc::gaussian_blur(
        &img, &mut background,
        Size::new(0, 0),
        sigma, sigma,
        core::BORDER_DEFAULT,
        core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    let mut diff = Mat::default();
    core::subtract(&img, &background, &mut diff, &core::no_array(), -1)?;

    let mut scratch_mask = Mat::default();
    imgproc::threshold(&diff, &mut scratch_mask, threshold, 255.0, imgproc::THRESH_BINARY)?;

    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_RECT, Size::new(3, 3), Point::new(-1, -1),
    )?;
    let mut dilated_mask = Mat::default();
    imgproc::dilate(
        &scratch_mask, &mut dilated_mask, &kernel,
        Point::new(-1, -1), 2,
        core::BORDER_CONSTANT,
        imgproc::morphology_default_border_value()?,
    )?;

    let mut inpainted = Mat::default();
    photo::inpaint(&img, &dilated_mask, &mut inpainted, inpaint_radius, photo::INPAINT_TELEA)?;

    let result = if denoise_strength > 0.0 {
        let mut denoised = Mat::default();
        photo::fast_nl_means_denoising(&inpainted, &mut denoised, denoise_strength, 7, 21)?;
        denoised
    } else {
        inpainted
    };

    // Exposure (EV stops) + contrast applied as a single linear transform:
    //   pixel = (2^exposure * contrast) * pixel + 128 * (1 - contrast)
    let final_result = if exposure != 0.0 || contrast != 1.0 {
        let alpha = 2f64.powf(exposure) * contrast;
        let beta = 128.0 * (1.0 - contrast);
        let mut adjusted = Mat::default();
        result.convert_to(&mut adjusted, -1, alpha, beta)?;
        adjusted
    } else {
        result
    };

    imgcodecs::imwrite(output_path, &final_result, &core::Vector::new())?;
    Ok(())
}

pub fn invert_negative(input_path: &str, output_path: &str) -> opencv::Result<()> {
    let img = imgcodecs::imread(input_path, imgcodecs::IMREAD_COLOR)?;
    if img.empty() {
        return Err(opencv::Error::new(core::StsError, format!("Could not read image: {}", input_path)));
    }
    let result = invert_mat(&img)?;
    imgcodecs::imwrite(output_path, &result, &core::Vector::new())?;
    Ok(())
}

fn invert_mat(img: &Mat) -> opencv::Result<Mat> {
    let mut inverted = Mat::default();
    core::bitwise_not(img, &mut inverted, &core::no_array())?;

    // Per-channel auto-levels to neutralise the orange mask in colour negatives
    let mut channels = core::Vector::<Mat>::new();
    core::split(&inverted, &mut channels)?;

    let mut normalised = core::Vector::<Mat>::new();
    for i in 0..3usize {
        let ch = channels.get(i)?;
        let mut norm = Mat::default();
        core::normalize(&ch, &mut norm, 0.0, 255.0, core::NORM_MINMAX, core::CV_8U, &core::no_array())?;
        normalised.push(norm);
    }

    let mut result = Mat::default();
    core::merge(&normalised, &mut result)?;
    Ok(result)
}
