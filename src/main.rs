#[macro_use]
extern crate diesel;
// doesn't work with use diesel_migrations::embed_migrations yet
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate bitflags;

mod cli;
mod database;
mod format;
mod filesystem;
mod functional;
mod linq;
mod os;
mod collections;

fn main() {
    println!("Hello, world!");
}
