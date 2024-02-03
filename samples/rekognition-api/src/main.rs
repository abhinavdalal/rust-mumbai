// refer: https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/

// setup api calling infra
// make photo-match api
// use the 2 s3 image urls to call rekognition api
// return true if face matches
// return false if its not

// TODO: handle special cases -> no urls etc

use dotenv::dotenv;
use std::env;

use actix_web::{
    get,
    // http::header::HeaderName,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder,
};

use awc::Client;

use aws_config::retry::RetryConfig;
// use aws_config;
use aws_sdk_rekognition as rekognition;
use rekognition::types::{Image, S3Object};
// use serde_json::{json, Value};

// can be in helper functions
fn get_s3_img_obj(bucket: &str, key: &str) -> Image {
    let s3_obj = S3Object::builder().bucket(bucket).name(key).build();
    Image::builder().s3_object(s3_obj).build()
}
// end - can be in helper functions

#[get("/face-match/{id}")]
async fn index(params: web::Path<String>) -> impl Responder {
    let _id = params.into_inner();
    // note: it is likely that you will use id here to get your keys via your database; we are just getting it from env variables for now
    let bucket = env::var("BUCKET").expect("bucket missing in env");
    let profile_key = env::var("PROFILE_KEY").expect("profile missing in env");
    let kyc_key = env::var("KYC_KEY").expect("kyc missing in env");
    let aws_env_config = aws_config::from_env()
        .retry_config(RetryConfig::disabled())
        .load()
        .await;
    let rekognition_client = rekognition::Client::new(&aws_env_config);
    let aws_res = rekognition_client
        .compare_faces()
        .set_source_image(Some(get_s3_img_obj(&bucket, &profile_key)))
        .set_target_image(Some(get_s3_img_obj(&bucket, &kyc_key)))
        .send()
        .await;

    match aws_res {
        Err(e) => {
            let err_msg = e.into_service_error();
            print!(
                "\nerror: {:?}\n",
                &err_msg.meta().message().expect("Unknown Error")
            );
            HttpResponse::ServiceUnavailable().body(format!(
                "{}",
                &err_msg.meta().message().expect("Unknown Error")
            ))
        }
        Ok(i) => {
            // print!("\naws_res: {:?}\n", &i);
            let face_matches = i.face_matches.expect("face matches error");
            if face_matches.len() != 1 {
                return HttpResponse::Ok().body("false\n\nface dont match or too many faces");
            }
            let match_percent = face_matches[0].similarity.expect("simirality not found");
            if match_percent < 98.0 {
                return HttpResponse::Ok().body("false\n\nface match not confident enough");
            }
            HttpResponse::Ok().body(format!("true\n\n{}", match_percent))
        }
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
async fn pong() -> impl Responder {
    HttpResponse::Ok().body("Pong")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(
                Client::builder()
                    .add_default_header((
                        "x-api-server-key",
                        env::var("API_SERVER_KEY").expect("no api server key in .env"),
                    ))
                    .finish(),
            ))
            .service(index)
            .service(hello)
            .route("/ping", web::get().to(pong))
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
