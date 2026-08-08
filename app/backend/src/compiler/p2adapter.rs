use std::path::Path;
use tokio::process::Command;

const ADAPTER_PATH: &str = "/app/wasi_snapshot_preview1.wasm";

pub async fn adapt_p1_to_p2(input_wasm: &Path, output_wasm: &Path) -> Result<(), String> {
    let output = Command::new("wasm-tools")
        .args([
            "component",
            "new",
            input_wasm.to_str().unwrap(),
            "-o",
            output_wasm.to_str().unwrap(),
            "--adapt",
            &format!("wasi_snapshot_preview1={}", ADAPTER_PATH),
        ])
        .output()
        .await
        .expect("Failed to execute P1>P2 adapter");

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}
