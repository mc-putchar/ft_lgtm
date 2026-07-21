mod compiler;
mod models;
mod runner;

use compiler::{CompileResult, compile_rust_to_wasm};
use models::{ApiResponse, ExecuteRequest, ExecuteResponse};
use runner::{RunResult, run_wasm};

use axum::routing::{get, post};
use axum::{Json, Router, extract::Path};
use std::env::var;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/v1/code/{cid}", get(poll_status))
        .route("/api/v1/execute", post(exec_code))
        .layer(CorsLayer::permissive());

    let host = var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = var("PORT").unwrap_or_else(|_| "3000".to_string());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    println!("Server running on http://{}:{}", host, port);

    axum::serve(listener, app).await.unwrap();
}

/// Fetch status of a code execution from the queue.
async fn poll_status(Path(cid): Path<String>) -> Json<ExecuteResponse> {
    println!("Polling status for CID: {}", cid);
    Json(ExecuteResponse {
        status: models::ExecutionStatus::Success,
        stdout: "Hello, World!".into(),
        stderr: "".into(),
        compilation_log: "".into(),
        execution_time_ms: 42,
        ipfs_cid: Some(cid.into()),
    })
}

/// Compiles code, runs it in Wasmtime, uploads results to IPFS, and returns the output.
async fn exec_code(Json(payload): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    println!(
        "Received code execution request for language: {}",
        payload.language
    );

    if payload.language != "rust" {
        return Json(ExecuteResponse {
            status: models::ExecutionStatus::CompilationError,
            stdout: "".into(),
            stderr: "Unsupported language".into(),
            compilation_log: "".into(),
            execution_time_ms: 0,
            ipfs_cid: None,
        });
    }

    println!("Compiling Rust code to WebAssembly...");

    let wasm_out: CompileResult = match compile_rust_to_wasm(&payload.code).await {
        Ok(wasm) => wasm,
        Err(e) => {
            return Json(ExecuteResponse {
                status: models::ExecutionStatus::CompilationError,
                stdout: "".into(),
                stderr: e,
                compilation_log: "".into(),
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    println!("Compilation finished. Success: {}", wasm_out.success);

    if wasm_out.success == false {
        return Json(ExecuteResponse {
            status: models::ExecutionStatus::CompilationError,
            stdout: "".into(),
            stderr: "Compilation failed".into(),
            compilation_log: wasm_out.logs,
            execution_time_ms: 0,
            ipfs_cid: None,
        });
    }

    println!("Running WebAssembly module...");

    let run_result = match run_wasm(&wasm_out.wasm_path).await {
        Ok(res) => res,
        Err(e) => {
            return Json(ExecuteResponse {
                status: models::ExecutionStatus::RuntimeError,
                stdout: "".into(),
                stderr: e,
                compilation_log: "".into(),
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    println!(
        "Execution finished. Time taken: {} ms",
        run_result.execution_time_ms
    );

    if let Some(error_message) = run_result.error_message {
        return Json(ExecuteResponse {
            status: models::ExecutionStatus::RuntimeError,
            stdout: run_result.stdout,
            stderr: error_message,
            compilation_log: "".into(),
            execution_time_ms: run_result.execution_time_ms,
            ipfs_cid: None,
        });
    }

    Json(ExecuteResponse {
        status: models::ExecutionStatus::Success,
        stdout: run_result.stdout,
        stderr: run_result.stderr,
        compilation_log: "".into(),
        execution_time_ms: run_result.execution_time_ms,
        ipfs_cid: Some("QmExampleCID".into()), // Placeholder for actual IPFS CID
    })
}
