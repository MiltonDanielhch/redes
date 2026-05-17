//! Ubicación: `apps/agent/src/main.rs`
//!
//! Descripción: Punto de entrada del agente de monitoreo.
//!              Recolecta métricas de red y las envía al servidor central.
//!
//! ADRs relacionados: 0022 (Agentes Distribuidos), 0020 (Monitoreo Regional)

use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();

    tracing::info!("Starting monitoring agent...");

    loop {
        tracing::info!("Agent running - collecting metrics");
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}