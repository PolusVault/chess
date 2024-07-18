use chess_backend::chess_server::ChessServer;
use chess_backend::config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::get_config().expect("Failed to parse configs");
    let chess_server = ChessServer::new(config)
        .build()
        .expect("Unable to build chess server");

    println!("Starting server...");

    // Start the server
    chess_server.await?;

    Ok(())
}
