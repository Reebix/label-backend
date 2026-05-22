extern crate qrc;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::mime::JSON;
use qrc::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct LabelInfo {
    name: String,
    qr: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/label")]
async fn label(json: web::Json<LabelInfo>) -> impl Responder {
    let qrcode = QRCode::from_string(json.qr.to_string());
    let png = qrcode.to_png(512);
    let png_data = png.into_raw();
    let png_image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(21, 21, png_data).unwrap();

    // Print the PNG representation of the QRCode
    println!("fn to_png(): {:?}", png_image.save("qrcode.png"));
    match png_image.save("qrcode.png") {
        // Print the path to the PNG representation of the QRCode that was saved to a file called "qrcode.png"
        Ok(_) => println!("png file created: qrcode.png"),
        // Print the path to the PNG representation of the QRCode that was saved to a file called "qrcode.png"
        Err(e) => println!("png file created: qrcode.png: {e}"),
    }
    std::fs::write("qrcode.png", png).expect("Unable to write file");
    HttpResponse::Ok().body( json.name.to_string() )
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(label)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}