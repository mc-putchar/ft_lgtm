use crate::compiler::p2adapter::adapt_p1_to_p2;
use crate::compiler::{COMPILE_TIMELIMIT, CompileResult, Compiler};

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, instrument};

pub struct CCompiler;

impl Compiler for CCompiler {
    /// Compiles C source code to WebAssembly component using `clang` + `wit-component`.
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        run_clang_pipeline(source_code, false).await
    }
}

pub struct CppCompiler;

impl Compiler for CppCompiler {
    /// Compiles C++ source code to WebAssembly component using `clang++` + `wit-component`.
    async fn compile(source_code: &str) -> Result<CompileResult, String> {
        run_clang_pipeline(source_code, true).await
    }
}

fn find_clang_binary(is_cpp: bool) -> String {
    let bin_name = if is_cpp { "clang++" } else { "clang" };

    if let Ok(wasi_path) = std::env::var("WASI_SDK_PATH") {
        let path = PathBuf::from(wasi_path).join("bin").join(bin_name);
        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    let homebrew = format!("/opt/homebrew/opt/llvm/bin/{bin_name}");
    if std::path::Path::new(&homebrew).exists() {
        return homebrew;
    }

    bin_name.to_string()
}

#[instrument(skip(source_code))]
async fn run_clang_pipeline(source_code: &str, is_cpp: bool) -> Result<CompileResult, String> {
    let workspace = TempDir::new().map_err(|e| format!("Failed to create temp dir: {e}"))?;
    let ext = if is_cpp { "cpp" } else { "c" };
    let compiler_cmd = find_clang_binary(is_cpp);
    info!(compiler_cmd = ?compiler_cmd, "Found compiler");

    let src_path = workspace.path().join(format!("main.{}", ext));
    let core_wasm_path = workspace.path().join("core.wasm");
    let component_wasm_path = workspace.path().join("out.wasm");

    tokio::fs::write(&src_path, source_code)
        .await
        .map_err(|e| format!("Failed to write source code: {e}"))?;

    // no_main builds as a reactor (equivalent to Rust's cdylib)
    let is_reactor = !source_code.contains("int main");

    let mut args = vec!["--target=wasm32-wasip1".to_string(), "-O3".to_string()];
    // Explicitly provide the sysroot so the linker finds crt1-reactor.o and libc
    if let Ok(wasi_path) = std::env::var("WASI_SDK_PATH") {
        let sysroot = PathBuf::from(wasi_path).join("share").join("wasi-sysroot");
        args.push(format!("--sysroot={}", sysroot.display()));
    }

    if is_reactor {
        args.push("-mexec-model=reactor".to_string());
        args.push("-Wl,--export-dynamic".to_string());
    }

    if is_cpp {
        args.push("-fno-exceptions".to_string());
        args.push("-fno-rtti".to_string());
    }

    args.push(src_path.to_str().unwrap().to_string());
    args.push("-o".to_string());
    args.push(core_wasm_path.to_str().unwrap().to_string());

    let compile_future = Command::new(&compiler_cmd).args(&args).output();

    let out = match timeout(Duration::from_secs(COMPILE_TIMELIMIT), compile_future).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return Err(format!("Clang execution failed: {e}")),
        Err(_) => {
            return Ok(CompileResult {
                success: false,
                wasm_path: component_wasm_path,
                logs: "Clang compilation timed out".into(),
                _workspace: workspace,
            });
        }
    };

    let mut logs = String::from_utf8_lossy(&out.stderr).into_owned();
    if !out.status.success() {
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
