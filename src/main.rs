use libavif_sys as c;
use minifb::{Window, WindowOptions};

fn main() {
    unsafe { unsafe_main() }
}

unsafe fn unsafe_main() {
    //
    // Decode AVIF
    //
    let avif_bytes = include_bytes!("../assets/frog.avif");

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
    let mut frames = Vec::with_capacity((*decoder).imageCount as usize);

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

        frames.push(buffer);
    }
    assert_eq!(frames.len(), (*decoder).imageCount as usize);
    c::avifDecoderDestroy(decoder);

    //
    // Windowing
    //
    let mut window = Window::new("debug-avif", width, height, WindowOptions::default()).unwrap();
    while window.is_open() {
        let buf: Vec<_> = frames[0]
            .chunks(4)
            .map(|rgba| {
                let [r, g, b, _] = *rgba else { unreachable!() };
                // TODO: Treat alpha properly
                (r as u32) << 16 | (g as u32) << 8 | (b as u32)
            })
            .collect();

        window.update_with_buffer(&buf, width, height).unwrap();
    }
}
