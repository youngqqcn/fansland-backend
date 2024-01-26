use base64::engine::general_purpose;
use base64::Engine;
use image::ImageFormat;
use qrcode::QrCode;
use std::{error::Error, io::Cursor};
use tracing::{info, warn};

pub fn get_qrcode_png_base64(qrcode_str: &str) -> String {
    // Encode some data into bits.
    tracing::info!("qrcode string: {}", qrcode_str);
    let code = QrCode::new(qrcode_str.as_bytes()).unwrap();

    // Render the bits into an image.
    let image = code
        .render::<image::Rgb<u8>>()
        .max_dimensions(170, 170)
        .quiet_zone(false)
        .build();

    // Save the image.
    image.save("./qrcode.png").unwrap();
    let mut buffer = Vec::new();
    if let Err(e) = image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png) {
        tracing::error!("write_to error:{} ", e.to_string());
    }

    let b64 = general_purpose::STANDARD.encode(buffer);
    tracing::info!("{b64}");

    b64
}
