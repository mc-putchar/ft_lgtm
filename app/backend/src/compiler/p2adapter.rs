use std::path::Path;
use wit_component::ComponentEncoder;

const ADAPTER_PATH: &str = "/opt/wasi_snapshot_preview1.command.wasm";
const REACTOR_PATH: &str = "/opt/wasi_snapshot_preview1.reactor.wasm";

pub async fn adapt_p1_to_p2(
    input_wasm: &Path,
    output_wasm: &Path,
    is_reactor: bool,
) -> Result<(), String> {
    let adapter_path = if is_reactor {
        REACTOR_PATH
    } else {
        ADAPTER_PATH
    };
    let adapter_bytes = match tokio::fs::read(adapter_path).await {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("Failed to read WASI adapter: {e}")),
    };

    let core_wasm = match tokio::fs::read(&input_wasm).await {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("Failed to read core wasm: {e}")),
    };

    let component_bytes = match ComponentEncoder::default()
        .module(&core_wasm)
        .map_err(|e| format!("Failed to configure ComponentEncoder: {e}"))
        .and_then(|enc| {
            // Apply the adapter to map the Preview 1 imports
            enc.adapter("wasi_snapshot_preview1", &adapter_bytes)
                .map_err(|e| format!("Failed to apply adapter: {e}"))
        })
        .and_then(|enc| {
            enc.validate(true)
                .encode()
                .map_err(|e| format!("Component encode failed: {e}"))
        }) {
        Ok(bytes) => bytes,
        Err(e) => return Err(e),
    };

    if let Err(e) = tokio::fs::write(&output_wasm, &component_bytes).await {
        return Err(format!("Failed to write component wasm: {e}"));
    }

    Ok(())
}
