use ab_glyph::{FontRef, PxScale};
use actix_files::NamedFile;
use clap;
use qrcode::QrCode;
use std::process::Command;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use imageproc::drawing::{draw_text_mut, text_size};
use imageproc::image::{ImageBuffer, Luma};
use serde::Deserialize;

#[derive(Deserialize)]
struct LabelInfo {
    name: String,
    qr: String,
}

#[get("/")]
async fn index() -> impl Responder {
    let page = include_str!("index.html");

    HttpResponse::Ok().content_type("text/html").body(page)
}

#[get("/last")]
async fn last() -> impl Responder {
    NamedFile::open("./label.png")
}

#[post("/preview")]
async fn preview(json: web::Json<LabelInfo>) -> impl Responder {
    let mut width = 1000;

    let bits = json.qr.clone();
    let code = QrCode::new(bits).unwrap();

    let qr = code
        .render::<Luma<u8>>()
        .quiet_zone(false)
        .max_dimensions(width, width)
        .build();

    let raw_qr = qr.as_raw();
    let qr_dim = qr.height();
    width = qr_dim;

    let mut label_image_buf = ImageBuffer::new(width, width * 2);

    let font = FontRef::try_from_slice(include_bytes!("../DejaVuSans.ttf")).unwrap();

    let mut s = width as f32;
    let mut scale = PxScale::from(s);
    while text_size(scale, &font, &json.name).0 > width {
        let new_size = s * 0.99;
        if new_size < 1.0 {
            break;
        }
        s = new_size;
        scale = PxScale::from(s);
    }

    let final_size = text_size(scale, &font, &json.name);
    let mut text_image = ImageBuffer::new(width, width * 2);

    draw_text_mut(
        &mut text_image,
        image::Rgb([255u8, 255u8, 255u8]),
        ((width - final_size.0) / 2) as i32,
        (width + width / 2 - final_size.1 / 2) as i32,
        scale,
        &font,
        &json.name,
    );
    let raw_text = text_image.pixels().collect::<Vec<_>>();

    let height = width * 2;

    label_image_buf
        .enumerate_pixels_mut()
        .for_each(|(x, y, pixel)| {
            if y < width {
                let mut raw = 255u8;
                let pos = y * width + x;
                if pos < raw_qr.len() as u32 {
                    raw = raw_qr[(x + width * y) as usize];
                }

                *pixel = image::Rgb([raw, raw, raw]);
            } else {
                let col = 255u8
                    - raw_text[(width - x + width * (height - y + width - 1) - 1) as usize].0[0];
                *pixel = image::Rgb([col, col, col]);
            }
        });

    label_image_buf.save("./label.png").unwrap();

    HttpResponse::Ok().body("Label created!")
}

#[post("/label")]
async fn label(json: web::Json<LabelInfo>) -> impl Responder {
    let mut width = 1000;

    let bits = json.qr.clone();
    let code = QrCode::new(bits).unwrap();

    let qr = code
        .render::<Luma<u8>>()
        .quiet_zone(false)
        .max_dimensions(width, width)
        .build();

    let raw_qr = qr.as_raw();
    let qr_dim = qr.height();
    width = qr_dim;

    let mut label_image_buf = ImageBuffer::new(width, width * 2);

    let font = FontRef::try_from_slice(include_bytes!("../DejaVuSans.ttf")).unwrap();

    let mut s = width as f32;
    let mut scale = PxScale::from(s);
    while text_size(scale, &font, &json.name).0 > width {
        let new_size = s * 0.99;
        if new_size < 1.0 {
            break;
        }
        s = new_size;
        scale = PxScale::from(s);
    }

    let final_size = text_size(scale, &font, &json.name);
    let mut text_image = ImageBuffer::new(width, width * 2);

    draw_text_mut(
        &mut text_image,
        image::Rgb([255u8, 255u8, 255u8]),
        ((width - final_size.0) / 2) as i32,
        (width + width / 2 - final_size.1 / 2) as i32,
        scale,
        &font,
        &json.name,
    );
    let raw_text = text_image.pixels().collect::<Vec<_>>();

    let height = width * 2;

    label_image_buf
        .enumerate_pixels_mut()
        .for_each(|(x, y, pixel)| {
            if y < width {
                let mut raw = 255u8;
                let pos = y * width + x;
                if pos < raw_qr.len() as u32 {
                    raw = raw_qr[(x + width * y) as usize];
                }

                *pixel = image::Rgb([raw, raw, raw]);
            } else {
                let col = 255u8
                    - raw_text[(width - x + width * (height - y + width - 1) - 1) as usize].0[0];
                *pixel = image::Rgb([col, col, col]);
            }
        });

    label_image_buf.save("./label.png").unwrap();

    let mut cmd = Command::new("lprint");

    cmd.arg("label.png").spawn().unwrap();

    HttpResponse::Ok().body("Label created!")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting server at http://{}:{}", args.host, args.port);

    HttpServer::new(|| {
        App::new()
            .service(label)
            .service(last)
            .service(index)
            .service(preview)
    })
    .bind((args.host.as_str(), args.port))?
    .run()
    .await
}
