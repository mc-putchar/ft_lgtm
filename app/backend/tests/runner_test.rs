use backend::compiler::Compiler;
use backend::compiler::rustc::RustCompiler;
use backend::runner::{ENGINE, run_wasm};

#[tokio::test]
async fn test_runner_valid_execution_and_output() {
    let code = r#"
        fn main() {
            println!("Output to stdout");
            eprintln!("Output to stderr");
        }
    "#;

    let compiled = RustCompiler::compile(code).await.unwrap();
    assert!(compiled.success, "Pre-test compilation failed");

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok(), "Execution failed unexpectedly");

    let res = run_res.unwrap();
    assert_eq!(res.stdout.trim(), "Output to stdout");
    assert_eq!(res.stderr.trim(), "Output to stderr");
    assert!(res.error_message.is_none(), "Expected no execution error");
}

#[tokio::test]
async fn test_runner_security_fuel_exhaustion() {
    let infinite_loop_code = r#"
        fn main() {
            loop {}
        }
    "#;

    let compiled = RustCompiler::compile(infinite_loop_code).await.unwrap();
    assert!(compiled.success, "Pre-test compilation failed");

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok());

    let res = run_res.unwrap();
    assert!(
        res.error_message.is_some(),
        "Runner should have caught fuel exhaustion error"
    );

    let err = res.error_message.unwrap();
    assert!(
        err.contains("fuel") || err.contains("trap"),
        "Error message should mention fuel/trap, got: {}",
        err
    );
}

#[tokio::test]
async fn test_runner_security_memory_limit() {
    let oom_code = r#"
        fn main() {
            let mut vec = Vec::new();
            vec.resize(60 * 1024 * 1024, 42u8);
            println!("Allocated {} bytes", vec.len());
        }
    "#;

    let compiled = RustCompiler::compile(oom_code).await.unwrap();
    assert!(compiled.success, "Pre-test compilation failed");

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok());

    let res = run_res.unwrap();
    assert!(
        res.error_message.is_some(),
        "Runner should trap when exceeding memory limits"
    );

    let err = res.error_message.unwrap();
    assert!(
        err.contains("memory")
            || err.contains("allocation")
            || err.contains("trap")
            || err.contains("unreachable")
            || err.contains("non-zero"),
        "Error should indicate a memory limits trap or allocation failure, got: {}",
        err
    );
}

#[tokio::test]
async fn test_runner_nonzero_exit_code() {
    let exit_code = r#"
        fn main() {
            std::process::exit(1);
        }
    "#;

    let compiled = RustCompiler::compile(exit_code).await.unwrap();
    assert!(compiled.success, "Pre-test compilation failed");

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok());

    let res = run_res.unwrap();
    assert_eq!(
        res.error_message,
        Some("Program exited with non-zero status code".to_string())
    );
}

#[tokio::test]
async fn test_runner_output_pipe_capacity() {
    let large_output_code = r#"
        fn main() {
            for _ in 0..1000 {
                println!("1234567890_1234567890_1234567890");
            }
        }
    "#;

    let compiled = RustCompiler::compile(large_output_code).await.unwrap();
    assert!(compiled.success);

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok());

    let res = run_res.unwrap();
    assert!(
        res.stdout.len() <= 10 * 1024,
        "Output pipe should cap stdout at 10KB, got {} bytes",
        res.stdout.len()
    );
}

#[tokio::test]
async fn test_runner_heavy_workload_within_limits() {
    let heavy_code = r#"
        fn main() {
            let size = 5 * 1024 * 1024; // 5 MB
            let mut data: Vec<u8> = vec![0; size];

            let mut state: u64 = 0x1234_5678_9ABC_DEF0;
            for i in 0..size {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                data[i] = (state & 0xFF) as u8;
            }

            let checksum: u64 = data
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_add(x as u64));

            println!("Processed {} bytes. Checksum: {}", data.len(), checksum);
        }
    "#;

    let compiled = RustCompiler::compile(heavy_code).await.unwrap();
    assert!(compiled.success, "Compilation failed: {}", compiled.logs);

    let run_res = run_wasm(&ENGINE, &compiled.wasm_path).await;
    assert!(run_res.is_ok(), "Execution wrapper failed");

    let res = run_res.unwrap();

    assert!(
        res.error_message.is_none(),
        "Program should have succeeded, but got error: {:?}",
        res.error_message
    );
    assert!(
        res.execution_time_ms > 0,
        "Execution time should be recorded"
    );
}
