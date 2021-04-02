#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;
use diesel::Connection;
use rocket::{Rocket, fairing::AdHoc, response::Redirect};
use rocket_contrib::json::Json;
use serde::Serialize;

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

// Database

#[database("logging")]
struct LoggingDatabaseConnection(diesel::SqliteConnection);

// JSON Responses

#[derive(Serialize)]
struct AdminStatistics {
    hits: u32
}

// Main Code

#[get("/admin/api/stats", format = "json")]
fn statistics() -> Json<AdminStatistics> {
    Json(AdminStatistics {
        hits: 0
    })
}

#[get("/<path..>")]
fn rick_roll(path: PathBuf, data: LoggingDatabaseConnection) -> Redirect {
    println!("Accessed {}", path.to_string_lossy());

    Redirect::to("https://www.youtube.com/watch?v=xvFZjo5PgG0")
}

fn create_structures(rocket: Rocket) -> Result<Rocket, Rocket> {
    match LoggingDatabaseConnection::get_one(&rocket) {
        Some(database) => {
            database.execute("CREATE TABLE IF NOT EXISTS hits ( \
                path TEXT, \
                time DATETIME DEFAULT CURRENT_TIMESTAMP, \
                ip_hash TEXT \
            )").expect("Database creation failed");

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