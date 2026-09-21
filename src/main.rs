use ab_glyph::{FontRef, PxScale};
use actix_files::NamedFile;
use std::env::temp_dir;

use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, post, web};
use clap;
use clap::Parser;
use imageproc::drawing::{draw_text_mut, text_size};
use imageproc::image::{ImageBuffer, Luma};
use qrcode::QrCode;
use serde::Deserialize;
use serde_json::json;
use std::process::Command;

#[derive(Deserialize)]
struct LabelInfo {
    name: String,
    qr: String,
    flipped: Option<bool>,
}

#[get("/")]
async fn index() -> impl Responder {
    let page = include_str!("index.html");

    HttpResponse::Ok().content_type("text/html").body(page)
}

#[get("/last")]
async fn last() -> impl Responder {
    let path = temp_dir().join("label.png");
    NamedFile::open(path).unwrap()
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}
#[post("/file")]
async fn file(MultipartForm(form): MultipartForm<UploadForm>) -> Result<impl Responder, Error> {
    for f in form.files {
        let path = temp_dir().join("label.png");

        std::fs::copy(f.file.path(), &path)?;

        Command::new("lprint").arg(&path).spawn()?;
    }

    Ok(HttpResponse::Ok())
}

fn gen_label_qr(json: web::Json<LabelInfo>) {
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
                let mut col = 255u8;

                if let Some(flipped) = json.flipped
                    && flipped
                {
                    col -=
                        raw_text[(width - x + width * (height - y + width - 1) - 1) as usize].0[0];
                } else {
                    col -= raw_text[(x + width * y) as usize].0[0];
                }

                *pixel = image::Rgb([col, col, col]);
            }
        });

    let path = temp_dir().join("label.png");
    label_image_buf.save(&path).unwrap();
}

fn gen_label(json: web::Json<LabelInfo>) {
    let width = 2000u32;
    let height = 1000u32;

    let font = FontRef::try_from_slice(include_bytes!("../DejaVuSans.ttf")).unwrap();

    let mut size = height as f32;
    let mut scale = PxScale::from(size);

    while text_size(scale, &font, &json.name).1 > height
        || text_size(scale, &font, &json.name).0 > width
    {
        size *= 0.99;

        if size < 0.01 {
            break;
        }

        scale = PxScale::from(size);
    }

    let final_size = text_size(scale, &font, &json.name);

    println!("final_size: {:?}", final_size);

    let mut text_image = ImageBuffer::new(width, height);

    let x = ((width as i32 - final_size.0 as i32) / 2).max(0);
    let y = ((height as i32 - final_size.1 as i32) / 2).max(0);

    draw_text_mut(
        &mut text_image,
        image::Rgb([255u8, 255u8, 255u8]),
        x,
        y,
        scale,
        &font,
        &json.name,
    );

    let mut label_image_buf = ImageBuffer::new(width, height);

    label_image_buf
        .enumerate_pixels_mut()
        .for_each(|(x, y, pixel)| {
            let src_x = if json.flipped.unwrap_or(false) {
                width - 1 - x
            } else {
                x
            };

            let src_y = if json.flipped.unwrap_or(false) {
                height - 1 - y
            } else {
                y
            };

            let raw_pixel = text_image.get_pixel(src_x, src_y);

            let col = 255u8.saturating_sub(raw_pixel.0[0]);

            *pixel = image::Rgb([col, col, col]);
        });

    let path = temp_dir().join("label.png");
    label_image_buf.save(&path).unwrap();
}

#[post("/preview")]
async fn preview(json: web::Json<LabelInfo>) -> impl Responder {
    if json.qr.is_empty() {
        gen_label(json);
    } else {
        gen_label_qr(json);
    }

    HttpResponse::Ok().body("Label created!")
}

#[post("/label")]
async fn label(json: web::Json<LabelInfo>) -> impl Responder {
    if json.qr.is_empty() {
        gen_label(json);
    } else {
        gen_label_qr(json);
    }

    let mut cmd = Command::new("lprint");
    let path = temp_dir().join("label.png");

    cmd.arg(&path).spawn().unwrap();

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
            .service(file)
    })
    .bind((args.host.as_str(), args.port))?
    .run()
    .await
}
