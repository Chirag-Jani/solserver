use actix_web::post;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::{signer::Signer, system_instruction};
use spl_token::instruction as token_instruction;

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
    from_address: String,
    to_address: String,
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
        &body.from_address.parse::<Pubkey>().unwrap(),
        &body.to_address.parse::<Pubkey>().unwrap(),
        body.lamports,
    );

    let response = SendSolResponse {
        success: true,
        data: SendSolData {
            program_id: body.from_address.clone(),
            accounts: transfer_instruction.accounts.clone(),
            instructions_data: transfer_instruction.clone(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendTokenResponse {
    success: bool,
    data: SendTokenData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SendTokenData {
    program_id: String,
    accounts: Vec<AccountMeta>,
    instructions_data: Instruction,
}

#[post("/send/token")]
async fn send_token(body: web::Json<SendTokenRequest>) -> impl Responder {
    let transfer_instruction = token_instruction::transfer(
        &spl_token::ID,                               // token program ID
        &body.owner.parse::<Pubkey>().unwrap(),       // source
        &body.destination.parse::<Pubkey>().unwrap(), // destination
        &body.mint.parse::<Pubkey>().unwrap(),        // authority
        &[&body.owner.parse::<Pubkey>().unwrap()],    // signer pubkeys
        body.amount,                                  // amount
    )
    .unwrap();

    let response = SendTokenResponse {
        success: true,
        data: SendTokenData {
            program_id: spl_token::ID.to_string(),
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VerifyMessageRequest {
    signature: String,
    message: String,
    pubkey: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VerifyMessageResponse {
    success: bool,
    data: VerifyMessageData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct VerifyMessageData {
    valid: bool,
    message: String,
    pubkey: String,
}

#[post("/verify/message")]
async fn verify_message(body: web::Json<VerifyMessageRequest>) -> impl Responder {
    let signature = body.signature.parse::<Signature>().unwrap();
    let message = body.message.as_bytes();
    let pubkey = body.pubkey.parse::<Pubkey>().unwrap();
    let valid = signature.verify(pubkey.as_ref(), message);

    let response = VerifyMessageResponse {
        success: true,
        data: VerifyMessageData {
            valid,
            message: body.message.clone(),
            pubkey: body.pubkey.clone(),
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
            .service(send_token)
            .service(sign_message)
            .service(verify_message)
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .await;
}
