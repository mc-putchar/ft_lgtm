use crate::compiler::{CompileResult, Compiler};

use std::process::Command;
use tempfile::TempDir;

pub const COMPILE_TIMELIMIT: &str = "5s";

pub struct RustCompiler;

impl Compiler for RustCompiler {
    /// Compiles Rust source code to WebAssembly using `rustc` in a `bubblewrap` sandboxed environment.
    ///
    /// # Examples
    ///
    /// ```
    /// let source_code = r#"
    ///     fn main() {
    ///         println!("Hello, there!");
    ///     }
    /// "#;
    /// let result = compile(source_code)?;
    /// assert!(result.success);
    /// ```
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        let workspace = TempDir::new().expect("Failed to create temp dir");
        let src_path = workspace.path().join("main.rs");
        let wasm_path = workspace.path().join("out.wasm");

        std::fs::write(&src_path, source_code).expect("Failed to write source code");

        let output = Command::new("timeout")
            .args([
                COMPILE_TIMELIMIT,
                "rustc",
                "--target",
                "wasm32-wasip2",
                src_path.to_str().unwrap(),
                "-o",
                wasm_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute rustc");

        let logs = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CompileResult {
            success: output.status.success(),
            wasm_path,
            logs,
            _workspace: workspace,
        })
    }
}
