use crate::compiler::{COMPILE_TIMELIMIT, CompileResult, Compiler, p2adapter};

use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::timeout;

pub struct CCompiler;

impl Compiler for CCompiler {
    /// Compiles C source code to WebAssembly using `zig cc`.
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        let workspace = TempDir::new().expect("Failed to create temp dir");
        let src_path = workspace.path().join("main.c");
        let raw_wasm = workspace.path().join("raw.wasm");
        let component_wasm = workspace.path().join("out.wasm");

        tokio::fs::write(&src_path, source_code)
            .await
            .expect("Failed to write source code");

        let compile_future = Command::new("zig")
            .args([
                "cc",
                "-target",
                "wasm32-wasi",
                src_path.to_str().unwrap(),
                "-o",
                raw_wasm.to_str().unwrap(),
            ])
            .output();

        let output = match timeout(Duration::from_secs(COMPILE_TIMELIMIT), compile_future).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(format!("Execution failed: {e}")),
            Err(_) => return Err("Compilation timed out".into()),
        };

        let logs = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Ok(CompileResult {
                success: false,
                wasm_path: raw_wasm,
                logs,
                _workspace: workspace,
            });
        }

        if let Err(e) = p2adapter::adapt_p1_to_p2(&raw_wasm, &component_wasm).await {
            return Ok(CompileResult {
                success: false,
                wasm_path: component_wasm,
                logs: format!("Component adaptation failed:\n{e}"),
                _workspace: workspace,
            });
        }

        Ok(CompileResult {
            success: output.status.success(),
            wasm_path: component_wasm,
            logs,
            _workspace: workspace,
        })
    }
}
