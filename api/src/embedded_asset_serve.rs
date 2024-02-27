use rust_embed::RustEmbed;
use actix_web::{get, Responder, web::{self, Bytes}, HttpResponse};
use log::trace;

#[derive(RustEmbed)]
#[folder = "src/public/"]
struct Asset;

#[get("/{tail:.*}")]
async fn serve_embedded( path: web::Path<(String,)> ) -> impl Responder {
    trace!("{:?}", path);


    match Asset::get(&path.0) {
        Some(file) => {
            let mut res = HttpResponse::Ok();
                
            // Content type
            if let Some(mime) = mime_guess::from_path(&path.0).first() {
                res.content_type(mime);
            }

            // Feed in embedded bytes
            res.body( Bytes::copy_from_slice(file.data.as_ref()) )
        },
        None => HttpResponse::NotFound().body("Static File not found!"), 
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/public")
            .service(serve_embedded)
            //.wrap(Authentication)
    );
}