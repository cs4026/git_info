#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate git_info;
extern crate rocket;
extern crate serde;
extern crate serde_json;

use std::{path::Path,fs};

#[macro_use]
extern crate serde_derive;

#[get("/<_username>/<repository>/<_tree>")]
fn get_repo(_username: String, repository: String, _tree: String) -> String {
    let full_repo_path = &format!("~/dev/{}",repository);
    let dir = Path::new(full_repo_path);

    if dir.is_dir(){
        //let git_

        //if<_tree> 
    } else {

    }

    String::from("hello")
}

fn main() {
    rocket::ignite().mount("/repo", routes![get_repo]).launch();
}
