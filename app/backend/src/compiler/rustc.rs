use crate::compiler::{COMPILE_TIMELIMIT, CompileResult, Compiler};

use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::timeout;

pub struct RustCompiler;

impl Compiler for RustCompiler {
    /// Compiles Rust source code to WebAssembly using `rustc`.
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        let workspace = TempDir::new().expect("Failed to create temp dir");
        let src_path = workspace.path().join("main.rs");
        let wasm_path = workspace.path().join("out.wasm");

        tokio::fs::write(&src_path, source_code)
            .await
            .expect("Failed to write source code");

        let mut args = vec![
            "--edition".to_string(),
            "2024".to_string(),
            "--target".to_string(),
            "wasm32-wasip2".to_string(),
        ];

        if !source_code.contains("fn main()") {
            args.push("--crate-type".to_string());
            args.push("cdylib".to_string());
        }

        args.push(src_path.to_str().unwrap().to_string());
        args.push("-o".to_string());
        args.push(wasm_path.to_str().unwrap().to_string());

        let compile_future = Command::new("rustc").args(&args).output();

        let output = match timeout(Duration::from_secs(COMPILE_TIMELIMIT), compile_future).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(format!("Execution failed: {e}")),
            Err(_) => {
                return Ok(CompileResult {
                    success: false,
                    wasm_path,
                    logs: "Compilation timed out".into(),
                    _workspace: workspace,
                });
            }
        };

        let logs = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CompileResult {
            success: output.status.success(),
            wasm_path,
            logs,
            _workspace: workspace,
        })
    }
}
