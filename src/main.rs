#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate serde_derive;


extern crate git_info;
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate rocket_cors;

use std::{path::Path,fs};
use rocket::request::Request;
use std::io::Cursor;
use std::env;

use rocket::response::{self, Response, Responder};
use rocket::http::{Status,Method,ContentType};


use rocket_cors::{AllowedOrigins, AllowedHeaders};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error404{
    message: String
}

impl<'r> Responder<'r> for Error404 {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let msg = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(Cursor::new(msg))
            .header(ContentType::Plain)
            .status(Status::NotFound)
            .ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error400{
    message: String
}

impl<'r> Responder<'r> for Error400 {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let msg = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(Cursor::new(msg))
            .header(ContentType::Plain)
            .status(Status::BadRequest)
            .ok()
    }
}

#[catch(404)]
fn not_found(req: &Request) -> Error404 { Error404{message : String::from("Tree not found") } }

#[get("/<_username>/<repository>/<_tree>")]
fn get_repo(_username: String, repository: String, _tree: String) -> Result<String,Error404> {
    for (key, value) in env::vars() {
    println!("{}: {}", key, value);
    }
    let git_path = env::var("GIT_PATH").unwrap();
    let user_path = &format!("{}/{}",git_path,_username);
    if Path::new(user_path).is_dir() {
        let repo_path =  &format!("{}/{}.git",user_path,repository);
        let tree = if _tree != "VOID"  { Some(_tree) } else { None };
        let repo_master = &format!("{}",repo_path);

        //if !Path::new(repo_master).is_file() { return Err(Error404{message : String::from("Repo Uninitialized")}); }

        if Path::new(repo_path).is_dir(){
            match git_info::go(repo_path.clone(),tree){
                Ok(files)=>{
                    let files = files;
                    Ok(serde_json::to_string_pretty(&*files.clone()).unwrap())
                },
                Err(err)=>{
                    let error = Error404{message : err};
                    Err(error)
                }
            }
        } else {
            println!("\n\n=====DATA=====\n\n ");

            let error = Error404{message : String::from("No directory found")};
            Err(error)
        }
    }else { Err(Error404{message: String::from("User not found")}) }
}

#[get("/<_username>/<repository>/<_branch>")]
fn get_branch_files(_username: String, repository: String,_branch:String) -> Result<String,Error404> {
    let git_path = env::var("GIT_PATH").unwrap();
    let user_path = &format!("{}/{}",git_path,_username);

    if Path::new(user_path).is_dir() {
        let repo_path =  &format!("{}/{}.git",user_path,repository);

        if Path::new(repo_path).is_dir(){
            if _branch=="master"{
                match git_info::go(repo_path.clone(),Some("VOID".to_owned())){
                    Ok(files)=>{
                        let files = files;
                        Ok(serde_json::to_string_pretty(&*files.clone()).unwrap())
                    },
                    Err(err)=>{
                        let error = Error404{message : err};
                        Err(error)
                    }
                }
            }else{
                match git_info::get_branch_files(repo_path.clone(),_branch){
                    Ok(files)=>{
                        let files = files;
                        Ok(serde_json::to_string_pretty(&*files.clone()).unwrap())
                    },
                    Err(err)=>{
                        let error = Error404{message : err};
                        Err(error)
                    }
                }
            }

        } else {
            println!("\n\n=====DATA=====\n\n ");

            let error = Error404{message : String::from("No directory found")};
            Err(error)
        }
    }else { Err(Error404{message: String::from("User not found")}) }
}


#[get("/<_username>/<repository>")]
fn get_branches(_username: String, repository: String)-> Result<String,Error404>{
    let git_path = env::var("GIT_PATH").unwrap();
    let user_path = &format!("{}/{}",git_path,_username);


        if Path::new(user_path).is_dir() {
            let repo_path =  &format!("{}/{}.git",user_path,repository);
            let repo_master = format!("{}",repo_path);

            if Path::new(repo_path).is_dir(){
                match git_info::get_branches(repo_master){
                    Ok(branches)=>{
                        let branches = branches;
                        Ok(serde_json::to_string_pretty(&*branches.clone()).unwrap())
                    },
                    Err(err)=>{
                        let error = Error404{message : String::from("Internal Error")};
                        Err(error)
                    }
                }
            } else {
                println!("\n\n=====DATA=====\n\n ");

                let error = Error404{message : String::from("No directory found")};
                Err(error)
            }
        }else { Err(Error404{message: String::from("User not found")}) }
}

fn main() {

    let allowed_origins = AllowedOrigins::all();

    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    };


    rocket::ignite()
    .mount("/repo", routes![get_repo])
    .mount("/branch",routes![get_branches,get_branch_files])
    .attach(options)
    .catch(catchers![not_found])
    .launch();
}
