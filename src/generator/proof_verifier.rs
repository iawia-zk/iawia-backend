use std::str;
use std::path;
use crate::utils::get_tmp_folder_path;
use tokio::process::Command;

pub struct ProofVerifier {
    uuid: uuid::Uuid,
    verification_key_path: String,
}

impl ProofVerifier {
    pub fn new(uuid: uuid::Uuid, verification_key_path: String) -> Self {
        ProofVerifier {
            uuid,
            verification_key_path,
        }
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid.clone()
    }

    pub async fn run(&self, rapid_snark_verifier_exe: &str) -> Result<String, String> {
        // Assume that the temporary folder contains "public_inputs.json" and "proof.json"
        let tmp_folder_path = get_tmp_folder_path(&self.uuid.to_string());
        let inputs_path = path::Path::new(&tmp_folder_path).join("public_inputs.json");
        let proof_path = path::Path::new(&tmp_folder_path).join("proof.json");

        if !inputs_path.exists() {
            return Err("Public inputs file does not exist".to_string());
        }
        if !proof_path.exists() {
            return Err("Proof file does not exist".to_string());
        }

        let output = Command::new(rapid_snark_verifier_exe)
            .arg(&self.verification_key_path)
            .arg(inputs_path)
            .arg(proof_path)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Verifier failed: {}",
                str::from_utf8(&output.stderr).unwrap_or("Unknown error")
            ));
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| e.to_string())?;

        Ok(stdout.to_string())
    }
}
