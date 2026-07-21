use std::path::Path;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

struct WasiState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

pub async fn run_wasm(wasm_path: &Path) -> Result<RunResult, String> {
    let mut config = Config::new();
    config.consume_fuel(true);
    // Limit memory to 50MB
    // config.static_memory_maximum_size(50 * 1024 * 1024);

    let engine = Engine::new(&config).expect("Failed to create Wasmtime engine");
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).expect("Failed to add WASI to linker");

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(10 * 1024);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(10 * 1024);

    let mut builder = WasiCtxBuilder::new();
    builder.stdout(stdout.clone()).stderr(stderr.clone());

    let mut store = Store::new(
        &engine,
        WasiState {
            ctx: builder.build(),
            table: ResourceTable::new(),
        },
    );

    store.set_fuel(100_000_000).expect("Failed to set fuel");

    // let module = Module::from_file(&engine, wasm_path).expect("Failed to create module");
    let component = Component::from_file(&engine, wasm_path).expect("Failed to create component");

    let instance = linker
        .instantiate_async(&mut store, &component)
        .await
        .expect("Failed to instantiate module");

    let start = std::time::Instant::now();
    let run_func = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .expect("No _start function found");
    let run_res = run_func.call_async(&mut store, ()).await;
    let execution_time_ms = start.elapsed().as_millis() as u64;
    let error_message = match run_res {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    };

    Ok(RunResult {
        stdout: String::from_utf8_lossy(&stdout.contents()).into_owned(),
        stderr: String::from_utf8_lossy(&stderr.contents()).into_owned(),
        error_message,
        execution_time_ms,
    })
}
