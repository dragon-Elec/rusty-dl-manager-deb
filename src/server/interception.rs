use std::{fs::OpenOptions, io::Write};

use poem::{handler, listener::TcpListener, post, web::Json, Route, Server};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Url {
    value: String,
}

#[handler]
fn get_url(res: Json<Url>) -> String {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("urls.txt")
        .expect("Unable to open file");

    let line = format!("{}\n", res.value);
    file.write_all(line.as_bytes())
        .expect("Unable to write to file");
    let text = format!("Received: {}", res.value);
    text
}

#[tokio::main]
pub async fn init_server() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let app = Route::new().at("/", post(get_url));
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
