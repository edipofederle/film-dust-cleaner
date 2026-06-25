use opencv::{
    core::{self, Mat},
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

    let background = gaussian_blur_pure(&img, sigma)?;

    let mut diff = Mat::default();
    core::subtract(&img, &background, &mut diff, &core::no_array(), -1)?;

    let mut scratch_mask = Mat::default();
    imgproc::threshold(&diff, &mut scratch_mask, threshold, 255.0, imgproc::THRESH_BINARY)?;

    let dilated_mask = dilate_3x3_pure(&scratch_mask, 2)?;

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

fn gaussian_blur_pure(img: &Mat, sigma: f64) -> opencv::Result<Mat> {
    let rows = img.rows() as usize;
    let cols = img.cols() as usize;
    let src = img.data_bytes()?;

    let half = (3.0 * sigma).ceil() as usize;
    let ksize = 2 * half + 1;
    let kernel: Vec<f64> = (0..ksize)
        .map(|i| {
            let x = i as f64 - half as f64;
            (-x * x / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let ksum: f64 = kernel.iter().sum();
    let kernel: Vec<f64> = kernel.iter().map(|v| v / ksum).collect();

    // Horizontal pass — accumulate into f32 to preserve precision
    let mut horiz = vec![0f32; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            let mut acc = 0f64;
            for (ki, &kv) in kernel.iter().enumerate() {
                let offset = ki as isize - half as isize;
                let sc = (c as isize + offset).clamp(0, cols as isize - 1) as usize;
                acc += kv * src[r * cols + sc] as f64;
            }
            horiz[r * cols + c] = acc as f32;
        }
    }

    // Vertical pass — write final u8 result
    let mut result_data = vec![0u8; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            let mut acc = 0f64;
            for (ki, &kv) in kernel.iter().enumerate() {
                let offset = ki as isize - half as isize;
                let sr = (r as isize + offset).clamp(0, rows as isize - 1) as usize;
                acc += kv * horiz[sr * cols + c] as f64;
            }
            result_data[r * cols + c] = acc.round().clamp(0.0, 255.0) as u8;
        }
    }

    let mut result = unsafe { Mat::new_rows_cols(rows as i32, cols as i32, core::CV_8UC1)? };
    result.data_bytes_mut()?.copy_from_slice(&result_data);
    Ok(result)
}

// 3×3 morphological dilation (max filter), run `iterations` times.
fn dilate_3x3_pure(mask: &Mat, iterations: i32) -> opencv::Result<Mat> {
    let rows = mask.rows() as usize;
    let cols = mask.cols() as usize;
    let mut src: Vec<u8> = mask.data_bytes()?.to_vec();
    let mut dst = vec![0u8; rows * cols];

    for _ in 0..iterations {
        for r in 0..rows {
            for c in 0..cols {
                let mut max_val = 0u8;
                for dr in -1isize..=1 {
                    for dc in -1isize..=1 {
                        let nr = (r as isize + dr).clamp(0, rows as isize - 1) as usize;
                        let nc = (c as isize + dc).clamp(0, cols as isize - 1) as usize;
                        max_val = max_val.max(src[nr * cols + nc]);
                    }
                }
                dst[r * cols + c] = max_val;
            }
        }
        std::mem::swap(&mut src, &mut dst);
    }

    let mut result = unsafe { Mat::new_rows_cols(rows as i32, cols as i32, core::CV_8UC1)? };
    result.data_bytes_mut()?.copy_from_slice(&src);
    Ok(result)
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
