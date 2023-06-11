use std::time::Instant;

use libavif_sys as c;
use minifb::{Window, WindowOptions};

fn main() {
    unsafe { unsafe_main() }
}

unsafe fn unsafe_main() {
    //
    // Decode AVIF
    //
    let avif_bytes = include_bytes!("../assets/alpha.avif");

    let decoder = c::avifDecoderCreate();
    match c::avifDecoderSetIOMemory(decoder, avif_bytes.as_ptr(), avif_bytes.len()) {
        c::AVIF_RESULT_OK => (),
        err => panic!("avifDecoderSetIOMemory() failed: {err}"),
    }

    match c::avifDecoderParse(decoder) {
        c::AVIF_RESULT_OK => (),
        err => panic!("avifDecoderParse() failed: {err}"),
    }

    let width = (*(*decoder).image).width as usize;
    let height = (*(*decoder).image).height as usize;
    let duration = (*decoder).duration;

    let count = (*decoder).imageCount as usize;
    let mut ptr_and_frames = Vec::with_capacity(count);

    dbg!(width, height, duration, count);

    // Parse frames loop
    loop {
        match c::avifDecoderNextImage(decoder) {
            c::AVIF_RESULT_OK => (),
            c::AVIF_RESULT_NO_IMAGES_REMAINING => break,
            err => panic!("avifDecoderNextImage() failed: {err}"),
        }

        let mut rgb = c::avifRGBImage::default();
        let p_rgb = &mut rgb as *mut c::avifRGBImage;

        c::avifRGBImageSetDefaults(p_rgb, (*decoder).image);
        rgb.format = c::AVIF_RGB_FORMAT_RGBA;
        rgb.depth = 8;
        rgb.rowBytes = 4 * width as u32;
        let mut buffer = vec![0u8; 4 * width * height];
        rgb.pixels = buffer.as_mut_ptr();

        match c::avifImageYUVToRGB((*decoder).image, p_rgb) {
            c::AVIF_RESULT_OK => (),
            err => panic!("avifImageYUVToRGB() failed: {err}"),
        }

        // Convert RGBA into 0RGB format
        let frame: Vec<_> = buffer
            .chunks(4)
            .map(|rgba| {
                let [r, g, b, a] = *rgba else { unreachable!() };
                let a = a as u32;
                let r = r as u32 * a / 0xFF;
                let g = g as u32 * a / 0xFF;
                let b = b as u32 * a / 0xFF;
                debug_assert!(r <= 0xFF);
                debug_assert!(g <= 0xFF);
                debug_assert!(b <= 0xFF);
                r << 16 | g << 8 | b
            })
            .collect();
        ptr_and_frames.push(((*decoder).imageTiming.pts, frame));
    }
    debug_assert_eq!(ptr_and_frames.len(), count);
    c::avifDecoderDestroy(decoder);

    //
    // Windowing
    //
    let mut window = Window::new("avif-practice", width, height, WindowOptions::default()).unwrap();
    let now = Instant::now();
    while window.is_open() {
        let elapsed = now.elapsed().as_secs_f64();
        let idx = ptr_and_frames.partition_point(|(pts, _)| *pts <= elapsed % duration) - 1;
        debug_assert!(idx < ptr_and_frames.len());

        window
            .update_with_buffer(&ptr_and_frames[idx].1, width, height)
            .unwrap();
    }
}
