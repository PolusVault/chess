use actix::{Actor, Addr};
use actix_files::NamedFile;
use actix_web::Responder;
use actix_web::{dev::Server, get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config::Settings;
use crate::websocket::{
    server::WsChessServer, servers::in_memory::InMemoryServer, session::SessionActor,
};

pub struct ChessServer {
    config: Settings,
}

impl ChessServer {
    pub fn new(config: Settings) -> Self {
        Self { config }
    }

    pub fn build(&self) -> Result<Server, Error> {
        let in_memory_state = InMemoryServer::default();
        let websocket_server = WsChessServer::new(in_memory_state).start();

        let host = self.config.app.host.as_str();
        let port = self.config.app.port;

        let player_count_limit: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));

        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(websocket_server.clone()))
                .app_data(web::Data::new(player_count_limit.clone()))
                .service(index)
                .service(file)
                .service(websocket)
                .service(health_check)
        })
        .bind((host, port))?
        .run();

        Ok(server)
    }
}

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("./dist/index.html").await.unwrap()
}

#[get("/assets/{filename:.*}")]
async fn file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(format!(
        "./dist/assets/{}",
        path.display()
    ))?)
}

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    ws_server: web::Data<Addr<WsChessServer<InMemoryServer>>>,
) -> Result<HttpResponse, Error> {
    // let mut player_count = player_count_limit.lock().unwrap();

    // if *player_count < 3 {
    //     *player_count += 1;
    return ws::start(SessionActor::new(ws_server.get_ref().clone()), &req, stream);
    // } else {
    //     return Err(ErrorBadRequest("player limit reached"));
    // }
}
