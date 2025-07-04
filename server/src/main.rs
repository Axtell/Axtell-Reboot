use std::io;

use axtell_server::server;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    server::serve().await
}
