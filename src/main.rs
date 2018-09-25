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
use std::env;

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
fn not_found(req: &Request) -> Error { Error{message : String::from("Tree not found") } }

#[get("/<_username>/<repository>/<_tree>")]
fn get_repo(_username: String, repository: String, _tree: String) -> Result<String,Error> {
    let git_path = env::var("GIT_PATH").unwrap();
    let user_path = &format!("{}/{}",git_path,_username);
    println!("USERNAME:  {:?} / {:?}",user_path,Path::new(user_path).is_dir());
    if Path::new(user_path).is_dir() {
        let repo_path =  &format!("{}/{}.git",user_path,repository);
        let tree = if _tree != "VOID"  { Some(_tree) } else { None };
        println!("USERNAME:  {:?} / {:?}",repo_path,Path::new(repo_path).is_dir());
        if Path::new(repo_path).is_dir(){
            match git_info::go(repo_path.clone(),tree){
                Ok(files)=>{
                    let files = files;
                    Ok(serde_json::to_string_pretty(&*files.clone()).unwrap())
                },
                Err(err)=>{
                    let error = Error{message : err};
                    Err(error)
                }
            }
        } else {
            println!("\n\n=====DATA=====\n\n ");

            let error = Error{message : String::from("No directory found")};
            Err(error)
        }
    }else { Err(Error{message: String::from("User not found")}) }




}

fn main() {
    rocket::ignite()
    .mount("/repo", routes![get_repo])
    .catch(catchers![not_found])
    .launch();
}
