
// doesn't work with use diesel_migrations::embed_migrations yet
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate diesel;

mod database;
mod functional;

fn main() {
    println!("Hello, world!");
}
