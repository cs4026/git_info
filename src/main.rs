#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate serde_derive;


extern crate git_info;
extern crate rocket;
extern crate serde;
extern crate serde_json;

use std::{path::Path,fs};
use rocket::request::Request;
use std::io::Cursor;

use rocket::response::{self, Response, Responder};
use rocket::http::Status;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error{
    message: String
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let msg = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(Cursor::new(msg))
            .status(Status::NotFound)
            .ok()
    }
}

#[catch(404)]
fn not_found(req: &Request) -> String { String::from("what") }

#[get("/<_username>/<repository>/<_tree>")]
fn get_repo(_username: String, repository: String, _tree: String) -> Result<String,Error> {
    let full_repo_path = &format!("/Users/carlos/{}",repository);
    let dir = Path::new(full_repo_path);
    println!("\n\n{:#?}---{:#?}\n\n ",full_repo_path,dir.is_dir());
    let tree = if _tree != "VOID"  { Some(_tree) } else { None };

    if dir.is_dir(){
        match git_info::go(full_repo_path.clone(),tree){
            Ok(files)=>{
                let files = files.unwrap();
                Ok(serde_json::to_string_pretty(&*files.clone()).unwrap())
            },
            Err(err)=>{
                let error = Error{message : err};
                Err(error)
            }
        }
        //if<_tree>
    } else {
        println!("\n\n=====DATA=====\n\n ");

        let error = Error{message : String::from("No directory found")};
        Err(error)
    }


}

fn main() {
    rocket::ignite()
    .mount("/repo", routes![get_repo])
    .catch(catchers![not_found])
    .launch();
}
