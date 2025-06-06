use poem::{handler, http::StatusCode, listener::TcpListener, post, web::Json, Route, Server};
use serde::Deserialize;
use std::sync::Mutex;

pub static SERVER_STATE: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[derive(Debug, Deserialize)]
struct Url {
    value: String,
}

#[handler]
fn get_url(res: Json<Url>) {
    let mut state = SERVER_STATE.try_lock().unwrap();
    state.push(res.value.clone());
}

#[handler]
fn handle_head() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
pub async fn init_server() -> Result<(), std::io::Error> {
    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "poem=debug");
    // }
    // tracing_subscriber::fmt::init();

    let app = Route::new().at("/", post(get_url).head(handle_head));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
