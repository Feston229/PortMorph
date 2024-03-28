#[cfg(test)]
mod test {
    use crate::{config::load_config, tcp::listener::PtmListener};
    use axum::{routing::get, Router};
    use std::time::Duration;
    use tokio::net::TcpListener;

    async fn base_http_test(config_path: &str) {
        async fn root() -> &'static str {
            "OK"
        }
        let app = Router::new().route("/", get(root));

        let axum_listener = TcpListener::bind("127.0.0.1:8080")
            .await
            .expect("Failed to bind listener");
        tokio::spawn(async move { axum::serve(axum_listener, app).await });

        let config = load_config(config_path)
            .await
            .expect("Failed to load config");
        let ptm_listener = PtmListener::new(config);
        tokio::spawn(async move { ptm_listener.start().await.expect("Error in ptm") });

        // Ensure that both axum and ptm are up
        tokio::time::sleep(Duration::from_secs(1)).await;
        let resp = reqwest::get("http://127.0.0.1:9000")
            .await
            .expect("Failed to get")
            .text()
            .await
            .expect("Failed to get text");
        assert_eq!(resp, "OK");
    }

    #[tokio::test]
    async fn http_test() {
        base_http_test("ptm_test.toml").await;
    }

    /*     #[tokio::test]
    async fn tls_test() {
        base_http_test("ptm_test_tls.toml").await;
    } */
}
