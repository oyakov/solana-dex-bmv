use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing::info;

pub fn init_metrics() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    let builder = PrometheusBuilder::new().with_http_listener(addr);
    
    builder.install().expect("failed to install Prometheus recorder");
    
    info!("Metrics server started on http://{}", addr);
}
