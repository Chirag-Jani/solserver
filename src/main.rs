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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ErrorResponse {
    success: bool,
    error: String,
}

impl ErrorResponse {
    fn new(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
        }
    }
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

#[post("/keypair")]
async fn keypair() -> impl Responder {
    let keypair = match Keypair::new() {
        keypair => keypair,
        #[allow(unreachable_patterns)]
        _ => {
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::new("Failed to generate new keypair"));
        }
    };

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
    let from_pubkey = match body.from_address.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid from_address: {}", e)));
        }
    };

    let to_pubkey = match body.to_address.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid to_address: {}", e)));
        }
    };

    let transfer_instruction =
        system_instruction::transfer(&from_pubkey, &to_pubkey, body.lamports);

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

#[post("/send/token")]
async fn send_token(body: web::Json<SendTokenRequest>) -> impl Responder {
    let owner = match body.owner.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid owner address: {}", e)));
        }
    };

    let destination = match body.destination.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Invalid destination address: {}",
                e
            )));
        }
    };

    let mint = match body.mint.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid mint address: {}", e)));
        }
    };

    let transfer_instruction = match token_instruction::transfer(
        &spl_token::ID,
        &owner,
        &destination,
        &mint,
        &[&owner],
        body.amount,
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Failed to create transfer instruction: {}",
                e
            )));
        }
    };

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
    if body.message.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse::new("Message cannot be empty"));
    }

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
    let signature = match body.signature.parse::<Signature>() {
        Ok(sig) => sig,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid signature: {}", e)));
        }
    };

    let pubkey = match body.pubkey.parse::<Pubkey>() {
        Ok(key) => key,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid public key: {}", e)));
        }
    };

    let message = body.message.as_bytes();
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateTokenResponse {
    success: bool,
    data: CreateTokenData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CreateTokenData {
    program_id: String,
    accounts: Vec<AccountMeta>,
    instructions_data: Instruction,
}

#[post("/token/create")]
async fn create_token(body: web::Json<CreateTokenRequest>) -> impl Responder {
    let mint_authority = match body.mint_authority.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid mint authority: {}", e)));
        }
    };

    let mint = match body.mint.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid mint address: {}", e)));
        }
    };

    let initialize_mint_instruction = match token_instruction::initialize_mint(
        &spl_token::ID,
        &mint,
        &mint_authority,
        None,
        body.decimals,
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Failed to create mint instruction: {}",
                e
            )));
        }
    };

    let response = CreateTokenResponse {
        success: true,
        data: CreateTokenData {
            program_id: spl_token::ID.to_string(),
            accounts: initialize_mint_instruction.accounts.clone(),
            instructions_data: initialize_mint_instruction.clone(),
        },
    };
    HttpResponse::Ok().json(response)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MintTokenResponse {
    success: bool,
    data: MintTokenData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MintTokenData {
    program_id: String,
    accounts: Vec<AccountMeta>,
    instructions_data: Instruction,
}

#[post("/token/mint")]
async fn mint_token(body: web::Json<MintTokenRequest>) -> impl Responder {
    let mint = match body.mint.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::new(format!("Invalid mint address: {}", e)));
        }
    };

    let destination = match body.destination.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Invalid destination address: {}",
                e
            )));
        }
    };

    let authority = match body.authority.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Invalid authority address: {}",
                e
            )));
        }
    };

    let mint_to_instruction = match token_instruction::mint_to(
        &spl_token::ID,
        &mint,
        &destination,
        &authority,
        &[&authority],
        body.amount,
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse::new(format!(
                "Failed to create mint instruction: {}",
                e
            )));
        }
    };

    let response = MintTokenResponse {
        success: true,
        data: MintTokenData {
            program_id: spl_token::ID.to_string(),
            accounts: mint_to_instruction.accounts.clone(),
            instructions_data: mint_to_instruction.clone(),
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
            .service(create_token)
            .service(mint_token)
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .await;
}
