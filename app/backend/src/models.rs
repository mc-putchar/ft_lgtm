use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/ExecuteRequest.ts")]
pub struct ExecuteRequest {
    pub language: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/ExecutionStatus.ts")]
pub enum ExecutionStatus {
    Success,
    CompilationError,
    RuntimeError,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/ExecuteResponse.ts")]
pub struct ExecuteResponse {
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub compilation_log: String,
    pub execution_time_ms: u64,
    pub ipfs_cid: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/types/RunResult.ts")]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub fuel_consumed: u64,
}
