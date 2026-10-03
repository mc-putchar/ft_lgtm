use crate::compiler::p2adapter::adapt_p1_to_p2;
use crate::compiler::{COMPILE_TIMELIMIT, CompileResult, Compiler};

use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::timeout;

pub struct GoCompiler;

impl Compiler for GoCompiler {
    /// Compiles Go source code to WebAssembly using `tinygo` and `wit-component`.
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        let workspace = TempDir::new().expect("Failed to create temp dir");
        let src_path = workspace.path().join("main.go");
        let core_wasm_path = workspace.path().join("core.wasm");
        let component_wasm_path = workspace.path().join("out.wasm");

        tokio::fs::write(&src_path, source_code)
            .await
            .map_err(|e| format!("Failed to write source code: {e}"))?;

        let is_reactor = !source_code.contains("func main(");

        let mut args = vec!["build".to_string(), "-target=wasi".to_string()];

        if is_reactor {
            // TinyGo requires c-shared to compile libraries/reactors that export functions
            args.push("-buildmode=c-shared".to_string());
        }

        args.push("-o".to_string());
        args.push(core_wasm_path.to_str().unwrap().to_string());
        args.push(src_path.to_str().unwrap().to_string());

        let compile_future = Command::new("tinygo").args(&args).output();

        let output = match timeout(Duration::from_secs(COMPILE_TIMELIMIT), compile_future).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(format!("TinyGo execution failed: {e}")),
            Err(_) => {
                return Ok(CompileResult {
                    success: false,
                    wasm_path: component_wasm_path,
                    logs: "Compilation timed out".into(),
                    _workspace: workspace,
                });
            }
        };

        let mut logs = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Ok(CompileResult {
                success: false,
                wasm_path: component_wasm_path,
                logs,
                _workspace: workspace,
            });
        }

        if let Err(e) = adapt_p1_to_p2(&core_wasm_path, &component_wasm_path, is_reactor).await {
            logs.push_str(&format!("Failed to adapt P1 to P2: {e}"));
            return Ok(CompileResult {
                success: false,
                wasm_path: component_wasm_path,
                logs,
                _workspace: workspace,
            });
        }

        Ok(CompileResult {
            success: true,
            wasm_path: component_wasm_path,
            logs,
            _workspace: workspace,
        })
    }
}
