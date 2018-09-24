#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate git_info;
extern crate rocket;

use std::{path::Path};

#[get("/repo/<username>/<repository>")]
fn get_repo(username: String, repository: String) -> String {

    String::from("hello")
}

fn main() {
    rocket::ignite().mount("/", routes![get_repo]).launch();
}
