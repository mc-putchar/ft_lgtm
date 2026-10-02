use crate::compiler::{CompileResult, get_compiled_code};
use crate::ipfs::publish_to_ipfs;
use crate::models::{ExecuteRequest, ExecuteResponse, ExecutionStatus};
use crate::runner::{ENGINE, run_wasm};

use axum::Json;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{KeyValue, global};
use std::sync::LazyLock;
use tracing::{Span, error, info, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;

// Cached metrics instruments to reuse in requests
struct AppMetrics {
    http_requests: Counter<u64>,
    executions: Counter<u64>,
    comp_errors: Counter<u64>,
    comp_failures: Counter<u64>,
    exec_errors: Counter<u64>,
    exec_failures: Counter<u64>,
    exec_successes: Counter<u64>,
    exec_time: Histogram<f64>,
    fuel_consumed: Histogram<u64>,
}

static METRICS: LazyLock<AppMetrics> = LazyLock::new(|| {
    let meter = global::meter("backend");
    AppMetrics {
        http_requests: meter.u64_counter("http_requests_total").build(),
        executions: meter.u64_counter("code_executions_total").build(),
        comp_errors: meter.u64_counter("compilation_errors_total").build(),
        comp_failures: meter.u64_counter("compilation_failures_total").build(),
        exec_errors: meter.u64_counter("execution_errors_total").build(),
        exec_failures: meter.u64_counter("wasm_execution_failures_total").build(),
        exec_successes: meter.u64_counter("execution_successes_total").build(),
        exec_time: meter.f64_histogram("execution_time_s").build(),
        fuel_consumed: meter.u64_histogram("fuel_consumed").build(),
    }
});

/// Compiles code, runs it in Wasmtime, uploads results to IPFS, and returns the output.
#[instrument(
    name = "http_request_exec",
    skip(payload),
    fields(
        language = %payload.language,
        trace_id = tracing::field::Empty,
        ipfs_cid = tracing::field::Empty,
        otel.status_code = tracing::field::Empty,
        otel.status_description = tracing::field::Empty
    )
)]
pub async fn exec_code(Json(payload): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    let span = Span::current();
    let trace_id = span.context().span().span_context().trace_id().to_string();
    span.record("trace_id", &trace_id);

    METRICS
        .http_requests
        .add(1, &[KeyValue::new("endpoint", "exec_code")]);
    let lang_attr = KeyValue::new("language", payload.language.clone());
    METRICS.executions.add(1, &[lang_attr.clone()]);

    info!(language = %payload.language, "Received code execution request");

    let wasm_out: CompileResult = match get_compiled_code(&payload.language, &payload.code).await {
        Ok(wasm_out) => {
            if !wasm_out.success {
                tracing::Span::current().record("otel.status_code", "ERROR");
                tracing::Span::current().record("otel.status_description", "Compilation error");
                warn!(logs = %wasm_out.logs, "Compilation error");
                METRICS.comp_errors.add(1, &[lang_attr]);
                return Json(ExecuteResponse {
                    status: ExecutionStatus::CompilationError,
                    stdout: "".into(),
                    stderr: "Compilation error".into(),
                    compilation_log: wasm_out.logs,
                    execution_time_ms: 0,
                    ipfs_cid: None,
                });
            }
            wasm_out
        }
        Err(e) => {
            tracing::error!("Compilation failed: {}", e);
            tracing::Span::current().record("otel.status_code", "ERROR");
            tracing::Span::current().record("otel.status_description", e.to_string());
            error!(error = %e, "Compilation handler failure");
            METRICS.comp_failures.add(1, &[lang_attr]);
            return Json(ExecuteResponse {
                status: ExecutionStatus::CompilationError,
                stdout: "".into(),
                stderr: "Compilation failed".into(),
                compilation_log: e,
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    info!("Compilation successful, executing WASM module");

    let run_result = match run_wasm(&ENGINE, &wasm_out.wasm_path).await {
        Ok(res) => {
            if let Some(err_msg) = res.error_message {
                warn!(error = %err_msg, duration_ms = res.execution_time_ms, "WASM execution runtime error");
                METRICS.exec_errors.add(1, &[]);
                tracing::Span::current().record("otel.status_code", "ERROR");
                tracing::Span::current().record("otel.status_description", err_msg.as_str());

                return Json(ExecuteResponse {
                    status: ExecutionStatus::RuntimeError,
                    stdout: res.stdout,
                    stderr: res.stderr + err_msg.as_str(),
                    compilation_log: wasm_out.logs,
                    execution_time_ms: res.execution_time_ms,
                    ipfs_cid: None,
                });
            }
            res
        }
        Err(e) => {
            error!(error = %e, "WASM execution task failure");
            METRICS.exec_failures.add(1, &[]);
            tracing::Span::current().record("otel.status_code", "ERROR");
            tracing::Span::current().record("otel.status_description", e.to_string());
            return Json(ExecuteResponse {
                status: ExecutionStatus::RuntimeError,
                stdout: "".into(),
                stderr: e,
                compilation_log: wasm_out.logs,
                execution_time_ms: 0,
                ipfs_cid: None,
            });
        }
    };

    METRICS
        .exec_time
        .record(run_result.execution_time_ms as f64 / 1000.0, &[]);
    METRICS
        .fuel_consumed
        .record(run_result.fuel_consumed as u64, &[]);
    info!(
        duration_ms = run_result.execution_time_ms,
        "WASM execution success"
    );
    info!(consumed_fuel = run_result.fuel_consumed, "Fuel consumed");
    METRICS.exec_successes.add(1, &[]);

    let ipfs_cid = match publish_to_ipfs(&payload.code, &run_result).await {
        Ok(cid) => {
            info!(cid = %cid, "Successfully published to IPFS");
            tracing::Span::current().record("ipfs_cid", &cid);
            Some(cid)
        }
        Err(e) => {
            error!(error = %e, "Failed to publish to IPFS");
            tracing::Span::current().record("otel.status_code", "ERROR");
            tracing::Span::current().record("otel.status_description", e.to_string());
            None
        }
    };

    Json(ExecuteResponse {
        status: ExecutionStatus::Success,
        stdout: run_result.stdout,
        stderr: run_result.stderr,
        compilation_log: wasm_out.logs,
        execution_time_ms: run_result.execution_time_ms,
        ipfs_cid,
    })
}
