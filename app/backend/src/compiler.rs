use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct CompileResult {
    pub success: bool,
    pub wasm_path: PathBuf,
    pub logs: String,
    // keep TempDir in scope until end of execution
    pub _workspace: TempDir,
}

/// Compiles Rust source code to WebAssembly using `rustc` in a sandboxed environment.
pub async fn compile_rust_to_wasm(source_code: &str) -> Result<CompileResult, String> {
    let workspace = TempDir::new().expect("Failed to create temp dir");
    let src_path = workspace.path().join("main.rs");
    let wasm_path = workspace.path().join("out.wasm");

    std::fs::write(&src_path, source_code).expect("Failed to write source code");

    if !Command::new("which")
        .arg("bwrap")
        .output()
        .expect("Failed to check for bwrap")
        .status
        .success()
    {
        return Err("bwrap is not installed. Please install bubblewrap.".to_string());
    }

    let output = Command::new("bwrap")
        .args([
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--ro-bind",
            "/etc-alternatives",
            "/etc-alternatives",
            "--bind",
            workspace.path().to_str().unwrap(),
            workspace.path().to_str().unwrap(),
            "--unshare-all",
            "--new-session",
            "rustc",
            "--target",
            "wasm32-wasi",
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
