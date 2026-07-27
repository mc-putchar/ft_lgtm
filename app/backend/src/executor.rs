use crate::compiler::{CompileResult, get_compiled_code};
use crate::ipfs::publish_to_ipfs;
use crate::models::{ExecuteRequest, ExecuteResponse, ExecutionStatus};
use crate::runner::{ENGINE, run_wasm};

use axum::Json;
use metrics::{counter, histogram};
use tracing::{error, info, warn};

/// Compiles code, runs it in Wasmtime, uploads results to IPFS, and returns the output.
pub async fn exec_code(Json(payload): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    info!(language = %payload.language, "Received code execution request");
    counter!("http_requests_total", "endpoint" => "exec_code").increment(1);
    counter!("code_executions_total", "language" => payload.language.clone()).increment(1);

    let wasm_out: CompileResult = match get_compiled_code(&payload.language, &payload.code).await {
        Ok(wasm) => wasm,
        Err(e) => {
            error!(error = %e, "Compilation handler failure");
            counter!("compilation_failures_total", "language" => payload.language.clone())
                .increment(1);
            return Json(ExecuteResponse {
                status: ExecutionStatus::CompilationError,
                stdout: "".into(),
                stderr: "".into(),
                compilation_log: e,
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    if !wasm_out.success {
        warn!(logs = %wasm_out.logs, "Compilation error");
        counter!("compilation_errors_total", "language" => payload.language.clone()).increment(1);
        return Json(ExecuteResponse {
            status: ExecutionStatus::CompilationError,
            stdout: "".into(),
            stderr: "Compilation failed".into(),
            compilation_log: wasm_out.logs,
            execution_time_ms: 0,
            ipfs_cid: None,
        });
    }

    info!("Compilation successful, executing WASM module");

    let run_result = match run_wasm(&ENGINE, &wasm_out.wasm_path).await {
        Ok(res) => res,
        Err(e) => {
            error!(error = %e, "WASM execution task failure");
            counter!("wasm_execution_failures_total").increment(1);
            return Json(ExecuteResponse {
                status: ExecutionStatus::RuntimeError,
                stdout: "".into(),
                stderr: e,
                compilation_log: wasm_out.logs,
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    histogram!("execution_time_ms").record(run_result.execution_time_ms as f64);

    if let Some(error_message) = run_result.error_message {
        warn!(error = %error_message, duration_ms = run_result.execution_time_ms, "WASM execution runtime error");
        counter!("execution_errors_total").increment(1);

        return Json(ExecuteResponse {
            status: ExecutionStatus::RuntimeError,
            stdout: run_result.stdout,
            stderr: run_result.stderr + error_message.as_str(),
            compilation_log: wasm_out.logs,
            execution_time_ms: run_result.execution_time_ms,
            ipfs_cid: None,
        });
    }

    info!(
        duration_ms = run_result.execution_time_ms,
        "WASM execution success"
    );
    counter!("execution_successes_total").increment(1);

    let ipfs_cid = match publish_to_ipfs(&payload.code, &run_result).await {
        Ok(cid) => {
            info!(cid = %cid, "Successfully published to IPFS");
            Some(cid)
        }
        Err(e) => {
            error!(error = %e, "Failed to publish to IPFS");
            None
        }
    };

    Json(ExecuteResponse {
        status: ExecutionStatus::Success,
        stdout: run_result.stdout,
        stderr: run_result.stderr,
        compilation_log: wasm_out.logs,
        execution_time_ms: run_result.execution_time_ms,
        ipfs_cid,
    })
}
