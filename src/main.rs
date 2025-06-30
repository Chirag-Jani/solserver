use actix_web::post;
use actix_web::{App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct KeypairData {
    #[serde(rename = "pubkey")]
    pubkey: String,
    secret_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct KeypairResponse {
    success: bool,
    data: KeypairData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TokenCreateResponse {
    success: bool,
    data: String,
}

#[post("/keypair")]
async fn keypair() -> impl Responder {
    let keypair = Keypair::new();
    let response = KeypairResponse {
        success: true,
        data: KeypairData {
            pubkey: keypair.pubkey().to_string(),
            secret_key: keypair.to_base58_string(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[post("/token/create")]
async fn token_create() -> impl Responder {
    let response = TokenCreateResponse {
        success: true,
        data: "Token created".to_string(),
    };
    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() {
    dotenv().ok();
    HttpServer::new(|| App::new().service(keypair).service(token_create))
        .bind("0.0.0.0:8080")
        .unwrap()
        .run()
        .await;
}
