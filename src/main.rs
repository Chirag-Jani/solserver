use actix_web::post;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::{signer::Signer, system_instruction};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct KeypairData {
    #[serde(rename = "pubkey")]
    pubkey: String,
    secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct KeypairResponse {
    success: bool,
    data: KeypairData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AccountData {
    pubkey: String,
    is_signer: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendSolData {
    program_id: String,
    accounts: AccountData,
    instructions_data: Instruction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendSolResponse {
    success: bool,
    data: SendSolData,
}

#[post("/keypair")]
async fn keypair() -> impl Responder {
    let keypair = Keypair::new();
    let response = KeypairResponse {
        success: true,
        data: KeypairData {
            pubkey: keypair.pubkey().to_string(),
            secret: keypair.to_base58_string(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[post("/send/sol")]
async fn send_sol(body: web::Json<SendSolRequest>) -> impl Responder {
    let transfer_instruction = system_instruction::transfer(
        &body.from.parse::<Pubkey>().unwrap(),
        &body.to.parse::<Pubkey>().unwrap(),
        body.lamports,
    );

    let response = SendSolResponse {
        success: true,
        data: SendSolData {
            program_id: body.from.clone(),
            accounts: AccountData {
                pubkey: body.to.clone(),
                is_signer: false,
            },
            instructions_data: transfer_instruction.clone(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() {
    dotenv().ok();
    let _ = HttpServer::new(|| App::new().service(keypair).service(send_sol))
        .bind("0.0.0.0:8080")
        .unwrap()
        .run()
        .await;
}
