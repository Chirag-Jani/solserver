use actix_web::post;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use solana_sdk::instruction::{AccountMeta, Instruction};
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
struct SendSolData {
    program_id: String,
    accounts: Vec<AccountMeta>,
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
            accounts: transfer_instruction.accounts.clone(),
            instructions_data: transfer_instruction.clone(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SignMessageResponse {
    success: bool,
    data: SignMessageData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SignMessageData {
    signature: String,
    public_key: String,
    message: String,
}

#[post("/sign/message")]
async fn sign_message(body: web::Json<SignMessageRequest>) -> impl Responder {
    let kp = Keypair::from_base58_string(&body.secret);

    let message = body.message.as_bytes();
    let signature = kp.sign_message(message);

    let response = SignMessageResponse {
        success: true,
        data: SignMessageData {
            signature: signature.to_string(),
            public_key: kp.pubkey().to_string(),
            message: body.message.clone(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() {
    dotenv().ok();
    let _ = HttpServer::new(|| {
        App::new()
            .service(keypair)
            .service(send_sol)
            .service(sign_message)
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .await;
}
