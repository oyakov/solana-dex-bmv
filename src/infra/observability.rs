use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

pub fn init_metrics() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));

    // Install the recorder and get a handle to render metrics
    // In 0.12.1, install_recorder() returns the handle.
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    // Spawn a manual listener to serve metrics with the mandatory Content-Type header
    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("Metrics server started on http://{}", addr);
                loop {
                    match listener.accept().await {
                        Ok((mut stream, _)) => {
                            let handle = handle.clone();
                            tokio::spawn(async move {
                                let body = handle.render();
                                let response = format!(
                                    "HTTP/1.1 200 OK\r\n\
                                     Content-Type: text/plain; version=0.0.4\r\n\
                                     Content-Length: {}\r\n\
                                     Connection: close\r\n\
                                     \r\n\
                                     {}",
                                    body.len(),
                                    body
                                );
                                if let Err(e) = stream.write_all(response.as_bytes()).await {
                                    error!("Failed to write metrics response: {}", e);
                                }
                            });
                        }
                        Err(e) => error!("Failed to accept metrics connection: {}", e),
                    }
                }
            }
            Err(e) => error!("Failed to bind metrics server to {}: {}", addr, e),
        }
    });
}
