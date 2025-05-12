use serde::Serialize;
use sqlx::{types::chrono, Decode};

use crate::types::ProofType;

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(type_name = "smallint")]
#[repr(i16)]
pub enum Status {
    Pending,
    WitnessGenerated,
    ProofGenerated,
    Failed,
}

impl Into<i32> for Status {
    fn into(self) -> i32 {
        match self {
            Status::Pending => 0,
            Status::WitnessGenerated => 1,
            Status::ProofGenerated => 2,
            Status::Failed => 3,
        }
    }
}

// CREATE TABLE IF NOT EXISTS proofs ( 
//     request_id UUID PRIMARY KEY,
//     proof_type SMALLINT NOT NULL,
//     status SMALLINT DEFAULT 0, 
//     circuit_name VARCHAR(255) NOT NULL,
//     onchain BOOLEAN NOT NULL, 
//     created_at TIMESTAMP WITH TIME ZONE,
//     witness_generated_at TIMESTAMP WITH TIME ZONE,
//     proof_generated_at TIMESTAMP WITH TIME ZONE, 
//     proof JSON,
//     public_inputs TEXT[],
//     reason TEXT, 
//     identifier VARCHAR(255)
// );

#[derive(Debug, Decode, Serialize, sqlx::FromRow)]
pub struct ProofPayload {
    pub request_id: uuid::Uuid,
    pub proof_type: ProofType,
    pub status: Status,
    pub circuit_name: String,
    pub onchain: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub witness_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub proof_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub proof: Option<serde_json::Value>,
    pub public_inputs: Option<Vec<String>>,
    pub reason: Option<String>,
    pub identifier: Option<String>,
}
