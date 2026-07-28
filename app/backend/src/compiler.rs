pub mod rust;

use rust::RustCompiler;

use std::path::PathBuf;
use tempfile::TempDir;

#[derive(Debug)]
pub struct CompileResult {
    pub success: bool,
    pub wasm_path: PathBuf,
    pub logs: String,
    // keep TempDir in scope until end of execution
    pub _workspace: TempDir,
}

impl Default for CompileResult {
    fn default() -> Self {
        Self {
            success: false,
            wasm_path: PathBuf::new(),
            logs: String::new(),
            _workspace: TempDir::new().unwrap(),
        }
    }
}

pub trait Compiler {
    fn compile(
        source: &str,
    ) -> impl std::future::Future<Output = Result<CompileResult, String>> + Send;
}

pub async fn get_compiled_code(lang: &str, source: &str) -> Result<CompileResult, String> {
    match lang {
        "rust" => RustCompiler::compile(source).await,
        _ => Err(format!("Unsupported language: {}", lang)),
    }
}

#[tokio::test]
async fn test_compiler_success_valid_rust() {
    let code = r#"
        fn main() {
            println!("Hello from Wasm test!");
        }
    "#;

    let result = RustCompiler::compile(code).await;
    assert!(result.is_ok(), "Compiler returned an unexpected Err");

    let compile_res = result.unwrap();
    assert!(
        compile_res.success,
        "Compilation failed: {}",
        compile_res.logs
    );
    assert!(
        compile_res.wasm_path.exists(),
        "WASM binary was not generated"
    );
}

#[tokio::test]
async fn test_compiler_syntax_error() {
    let invalid_code = r#"
        fn main() {
            let x = ; // Invalid syntax
        }
    "#;

    let result = RustCompiler::compile(invalid_code).await;
    assert!(result.is_ok(), "Compiler task failed unexpectedly");

    let compile_res = result.unwrap();
    assert!(!compile_res.success, "Compilation should have failed");
    assert!(
        compile_res.logs.contains("error") || compile_res.logs.contains("expected"),
        "Compilation log should contain compiler error details"
    );
}

#[tokio::test]
async fn test_compiler_enforces_timeout() {
    let macro_bomb_code = r#"
        #![allow(long_running_const_eval)]
        const _: () = loop {};
        fn main() {}
    "#;

    let start = std::time::Instant::now();
    let result = RustCompiler::compile(macro_bomb_code).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    let compile_res = result.unwrap();

    assert!(
        !compile_res.success,
        "Compilation should fail due to timeout"
    );
    assert!(
        elapsed.as_secs() >= 5 && elapsed.as_secs() <= 8,
        "Compiler should terminate around the 5s timeout mark (took {}s)",
        elapsed.as_secs()
    );
}

#[tokio::test]
async fn test_get_compiled_code_unsupported_language() {
    let result = get_compiled_code("python", "print('hello')").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unsupported language: python");
}
