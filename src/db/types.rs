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
