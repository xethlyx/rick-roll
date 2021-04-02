#![feature(proc_macro_hygiene, decl_macro)]

use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, net::SocketAddr, path::PathBuf};
use rand::{distributions::Alphanumeric, Rng};
use rocket::{Rocket, fairing::AdHoc, response::Redirect};
use rocket_contrib::json::Json;
use serde::Serialize;

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

// Database

#[database("logging")]
struct LoggingDatabaseConnection(rusqlite::Connection);

// JSON Responses

#[derive(Serialize)]
struct AdminStatistics {
    hits: u32,
    hits_unique: u32
}

// Main Code

#[get("/stats", format = "text/html")]
fn statistics(data: LoggingDatabaseConnection) -> Json<AdminStatistics> {
    let hits: u32 = data.0.query_row("SELECT COUNT(*) FROM hits", &[], |row| row.get(0)).unwrap();
    let hits_unique: u32 = data.0.query_row("SELECT COUNT(DISTINCT ip_hash) FROM hits", &[], |row| row.get(0)).unwrap();

    Json(AdminStatistics {
        hits,
        hits_unique
    })
}

#[get("/<path..>", format = "text/html")]
fn rick_roll(path: PathBuf, data: LoggingDatabaseConnection, ip: SocketAddr) -> Redirect {
    let mut hasher = DefaultHasher::new();
    ip.ip().to_string().hash(&mut hasher);

    match data.0.execute("INSERT INTO hits (path, ip_hash) VALUES (?, ?)", &[&path.to_string_lossy(), &hasher.finish().to_string()]) {
        Ok(_) => (),
        Err(_) => println!("Error occurred while logging rick roll")
    }

    Redirect::to("https://www.youtube.com/watch?v=xvFZjo5PgG0")
}

fn create_structures(rocket: Rocket) -> Result<Rocket, Rocket> {
    match LoggingDatabaseConnection::get_one(&rocket) {
        Some(database) => {
            database.execute("CREATE TABLE IF NOT EXISTS hits ( \
                path TEXT, \
                time DATETIME DEFAULT CURRENT_TIMESTAMP, \
                ip_hash TEXT \
            )", &[]).expect("Database creation failed");

            database.execute("CREATE TABLE IF NOT EXISTS meta ( \
                key TEXT UNIQUE NOT NULL, \
                value TEXT \
            )", &[]).expect("Database creation failed");

            let secret: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();

            let _ = database.execute("INSERT INTO meta (key, value) VALUES ('secret', ?)", &[&secret]);

            Ok(rocket)
        }
        None => Err(rocket)
    }
}

fn main() {
    rocket::ignite()
        .attach(LoggingDatabaseConnection::fairing())
        .attach(AdHoc::on_attach("Create Structures", create_structures))
        .mount("/", routes![statistics, rick_roll])
        .launch();
}