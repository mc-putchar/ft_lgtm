use crate::models::RunResult;

use std::path::Path;
use std::sync::LazyLock;
use tracing::{Span, instrument};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Cache, CacheConfig, Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap};
use wasmtime_wasi::p2::bindings::Command;
use wasmtime_wasi::{I32Exit, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

const DEFAULT_FUEL: u64 = 50_000_000_000;
const DEFAULT_STDOUT_LIMIT: usize = 10 * 1024;
const DEFAULT_STDERR_LIMIT: usize = 10 * 1024;
const DEFAULT_MEMORY_SIZE_LIMIT: usize = 64 * 1024 * 1024;

struct WasiState {
    ctx: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

pub static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut config = Config::new();
    config.consume_fuel(true);

    if let Ok(cache) = Cache::new(CacheConfig::new()) {
        config.cache(Some(cache));
    }

    Engine::new(&config).expect("Failed to create Wasmtime engine")
});

#[instrument(
    name = "execute_wasm",
    skip(engine, wasm_path),
    fields(
        fuel_consumed = tracing::field::Empty,
        execution_time_ms = tracing::field::Empty
    ),
    err
)]
pub async fn run_wasm(engine: &Engine, wasm_path: &Path) -> Result<RunResult, String> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker).expect("Failed to add WASI to linker");

    let stdout = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(DEFAULT_STDOUT_LIMIT);
    let stderr = wasmtime_wasi::p2::pipe::MemoryOutputPipe::new(DEFAULT_STDERR_LIMIT);

    let mut builder = WasiCtxBuilder::new();
    builder.stdout(stdout.clone()).stderr(stderr.clone());

    let limits = StoreLimitsBuilder::new()
        .memory_size(DEFAULT_MEMORY_SIZE_LIMIT)
        .build();

    let mut store = Store::new(
        &engine,
        WasiState {
            ctx: builder.build(),
            table: ResourceTable::new(),
            limits,
        },
    );

    store.limiter(|state| &mut state.limits);
    store.set_fuel(DEFAULT_FUEL).expect("Failed to set fuel");

    let engine_clone = engine.clone();
    let wasm_path_clone = wasm_path.to_path_buf();

    let current_span = tracing::Span::current();
    let component = tokio::task::spawn_blocking(move || {
        let _entered = current_span.entered();
        Component::from_file(&engine_clone, &wasm_path_clone)
    })
    .await
    .expect("Failed to spawn blocking task")
    .expect("Failed to load component");

    let start = std::time::Instant::now();

    let command = Command::instantiate_async(&mut store, &component, &linker)
        .await
        .expect("Failed to instantiate command");

    let run_res = command.wasi_cli_run().call_run(&mut store).await;

    let execution_time_ms = start.elapsed().as_millis() as u64;
    let fuel_remaining = store.get_fuel().expect("Failed to get fuel");
    let fuel_consumed = DEFAULT_FUEL - fuel_remaining;

    let span = Span::current();
    span.record("fuel_consumed", fuel_consumed);
    span.record("execution_time_ms", execution_time_ms);

    let error_message = match run_res {
        Ok(Ok(())) => None,
        Ok(Err(())) => Some("Program exited with non-zero status code".to_string()),
        Err(e) => {
            if let Some(exit) = e.downcast_ref::<I32Exit>() {
                if exit.0 == 0 {
                    None
                } else {
                    Some("Program exited with non-zero status code".to_string())
                }
            } else if let Some(Trap::OutOfFuel) = e.downcast_ref::<Trap>() {
                Some("Execution limit exceeded: out of fuel".to_string())
            } else {
                Some(format!("{e:#}"))
            }
        }
    };

    Ok(RunResult {
        stdout: String::from_utf8_lossy(&stdout.contents()).into_owned(),
        stderr: String::from_utf8_lossy(&stderr.contents()).into_owned(),
        error_message,
        execution_time_ms,
        fuel_consumed,
    })
}
