use moteur_ia::web_server::run_web_server;

#[tokio::main]
async fn main() {
    run_web_server().await;
}