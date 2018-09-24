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
    let full_repo_path = &format!("/Users/carlos/{}",repository);
    let dir = Path::new(full_repo_path);
    println!("\n\n{:#?}---{:#?}\n\n ",full_repo_path,dir.is_dir());
    let tree = if _tree != "VOID"  { Some(_tree) } else { None };

    if dir.is_dir(){
        let information = git_info::go(full_repo_path.clone(),tree).unwrap();
        let data = serde_json::to_string_pretty(&*information.clone()).unwrap();

        data
        //if<_tree>
    } else {
        println!("\n\n=====DATA=====\n\n ");
        String::from("No data found")
    }


}

fn main() {
    rocket::ignite().mount("/repo", routes![get_repo]).launch();
}
