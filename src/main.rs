use libavif_sys as c;

fn main() {
    unsafe { avif() }
}

unsafe fn avif() {
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

    let width = (*(*decoder).image).width;
    let height = (*(*decoder).image).height;
    let mut frames = Vec::with_capacity((*decoder).imageCount as usize);

    // Parse frames loop
    loop {
        let mut buffer = vec![0u8; 4 * width as usize * height as usize];

        match c::avifDecoderNextImage(decoder) {
            c::AVIF_RESULT_OK => (),
            c::AVIF_RESULT_NO_IMAGES_REMAINING => break,
            err => panic!("avifDecoderNextImage() failed: {err}"),
        }

        let mut rgb = c::avifRGBImage {
            ..Default::default()
        };
        let p_rgb = &mut rgb as *mut c::avifRGBImage;

        c::avifRGBImageSetDefaults(p_rgb, (*decoder).image);
        rgb.format = c::AVIF_RGB_FORMAT_RGBA;
        rgb.depth = 8;
        rgb.rowBytes = 4 * width;
        rgb.pixels = buffer.as_mut_ptr();

        match c::avifImageYUVToRGB((*decoder).image, p_rgb) {
            c::AVIF_RESULT_OK => (),
            err => panic!("avifImageYUVToRGB() failed: {err}"),
        }

        frames.push(buffer);
    }
    assert_eq!(frames.len(), (*decoder).imageCount as usize);

    c::avifDecoderDestroy(decoder);
}
