//#![deny(warnings)]

extern crate git2;
extern crate serde;
extern crate serde_json;

use git2::{Commit, Oid, Repository, Tree, TreeEntry};
use std::path::Path;
use std::process::Command;



#[macro_use]
extern crate serde_derive;

#[derive(Debug)]
struct GitCommit {
    file: String,
    commit: String,
    author: String,
    time: String,
    msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    name: String,
    oid: String,
    commit: String,
    blob_type: String,
    author: String,
    time: i64,
    msg: String,
}

/**
* Function opens a repo and returns the repo object
**/
fn open_repo(path: String) -> Result<Repository, git2::Error> {
    let repo_path = Path::new(&path);
    Repository::open(repo_path)
}

fn clean_tree(tree: Option<String>)->Result<Option<Oid>,String>{
    match tree{
        Some(tree)=>{
            match Oid::from_str(&tree) {
                Ok(tree_oid)=>{ Ok(Some(tree_oid)) }, Err(e)=>{ Err(String::from("Tree ID not valid oid")) }
            }
        },
        None=>{ Ok(None) }
    }
}

/**
* Function is main entry point to library
**/
pub fn go(path: String,tree: Option<String>)->Result<Box<Vec<Box<File>>>,String>{

    let repo = open_repo(path).unwrap();
    //let tree: Option<Oid> = if tree.is_some() { Oid::from_str(&tree.unwrap()).ok() } else { None };

    let tree_oid = clean_tree(tree)?;

    match get_info(tree_oid,repo){
        Ok(files)=>{
            Ok(files)
        },
        Err(msg)=>{
            Err("Internal Error".to_string())
        }
    }

}

/**
* Function runs query for files.
**/
fn get_info(
    parent_tree: Option<Oid>,
    repo: Repository,
) -> Result<Box<Vec<Box<File>>>, git2::Error> {

    let mut walk = repo.revwalk()?;
    walk.push_head()?;

    let oids: Vec<::Oid> = walk.by_ref().collect::<Result<Vec<_>, _>>()?;

    //------part 0.5: Turn commit oids into commit blobs then convert to trees-------//
    let commits = get_commits(&oids, &repo);

    //Overhead associated with moving this to function too high.
    let trees = commits
        .iter()
        .map(|commit| commit.tree().unwrap())
        .collect();

    let file_tree: Tree;

    match parent_tree {
        Some(tree) => {

            file_tree = repo.find_tree(tree)?;
        }
        None => {
            let master = repo.find_commit(oids[0])?;
            file_tree = master.tree()?;
        }
    }

    let mut files = Box::new(get_files(&file_tree));
    add_commit(&mut files, &commits, &trees, parent_tree, &repo);

    Ok(files)
}

fn split(path: &str) -> Vec<String> {
    let mut path_args: Vec<String> = path.split("/").map(|arg| String::from(arg)).collect();
    path_args[0] = format!("{}/", path_args[0]);
    path_args
}

fn get_file_list(path: &str) -> Vec<Box<File>> {
    let repo_path;
    if &path[0..1] == "." {
        let ref path_args = split(path);
        repo_path = path_args[0].clone();
    } else {
        repo_path = String::from(path);
    }

    let repo = open_repo(repo_path).unwrap();

    let mut walk = repo.revwalk().unwrap();
    walk.push_head().unwrap();

    let oids: Vec<::Oid> = walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap();

    let master = repo.find_commit(oids[0]).unwrap();
    let master_tree = master.tree().unwrap();

    master_tree
        .iter()
        .map(|blob| {
            Box::new(File {
                name: String::from(blob.name().unwrap()),
                oid: format!("{}", blob.id()),
                commit: format!("{}", Oid::zero()),
                blob_type: String::from(blob.kind().unwrap().str()),
                author: String::from(""),
                time: 0,
                msg: String::from(""),
            })
        }).collect()
}

fn get_files(tree: &git2::Tree) -> Vec<Box<File>> {
    tree.iter()
        .map(|blob| {
            Box::new(File {
                name: String::from(blob.name().unwrap()),
                oid: format!("{}", blob.id()),
                commit: format!("{}", Oid::zero()),
                blob_type: String::from(blob.kind().unwrap().str()),
                author: String::from(""),
                time: 0,
                msg: String::from(""),
            })
        }).collect()
}

fn add_commit<'a>(
    files: &'a mut Vec<Box<File>>,
    commits: &'a Vec<Commit<'a>>,
    trees: &'a Vec<Tree<'a>>,
    parent_tree: Option<Oid>,
    repo: &Repository,
) {
    match parent_tree {
        Some(_tree) => {
            files.iter_mut().for_each(|mut file| {
                let end = commits.len();
                bsearch_deep(0, end, &mut file, trees, commits, repo);
            });
        }
        None => {
            files.iter_mut().for_each(|mut file| {
                let end = commits.len();
                bsearch(0, end, &mut file, trees, commits);
            });
        }
    }
}

fn bsearch(
    start: usize,
    end: usize,
    file: &mut Box<File>,
    trees: &Vec<Tree>,
    commits: &Vec<Commit>,
) {
    if trees.len() == 0 {
        return;
    }

    if trees.len() == 1 {
        file.commit = format!("{}", commits[0].id());
        file.author = String::from(commits[0].committer().name().unwrap());
        file.time = commits[0].committer().when().seconds();
        file.msg = String::from(commits[0].summary().unwrap());
        return;
    }

    let diff = if start > end {
        start - end
    } else {
        end - start
    };
    let mid = (start + end) / 2;

    if diff == 1 {
        match trees[end].get_id(Oid::from_str(&file.oid).unwrap()) {
            Some(_finally) => {
                file.commit = format!("{}", commits[end].id());
                file.author = String::from(commits[end].committer().name().unwrap());
                file.time = commits[end].committer().when().seconds();
                file.msg = String::from(commits[end].summary().unwrap());
            }
            None => {
                file.commit = format!("{}", commits[start].id());
                file.author = String::from(commits[start].committer().name().unwrap());
                file.time = commits[start].committer().when().seconds();
                file.msg = String::from(commits[start].summary().unwrap());
            }
        }
        return;
    }

    match trees[mid].get_id(Oid::from_str(&file.oid).unwrap()) {
        Some(_file1) => match trees[mid + 1].get_id(Oid::from_str(&file.oid).unwrap()) {
            Some(_file2) => {
                bsearch(mid + 1, end, file, trees, commits);
            }
            None => {
                file.commit = format!("{}", commits[mid].id());
                file.author = String::from(commits[mid].committer().name().unwrap());
                file.time = commits[mid].committer().when().seconds();
                file.msg = String::from(commits[mid].summary().unwrap());
                return;
            }
        },
        None => match trees[mid - 1].get_id(Oid::from_str(&file.oid).unwrap()) {
            Some(_file2) => {
                file.commit = format!("{}", commits[mid - 1].id());
                file.author = String::from(commits[mid - 1].committer().name().unwrap());
                file.time = commits[mid - 1].committer().when().seconds();
                file.msg = String::from(commits[mid - 1].summary().unwrap());
                return;
            }
            None => {
                bsearch(start, mid - 1, file, trees, commits);
            }
        },
    }
}

fn bsearch_deep(
    start: usize,
    end: usize,
    file: &mut Box<File>,
    trees: &Vec<Tree>,
    commits: &Vec<Commit>,
    repo: &Repository,
) {
    if trees.len() == 0 {
        return;
    }

    if trees.len() == 1 {
        file.commit = format!("{}", commits[0].id());
        file.author = String::from(commits[0].committer().name().unwrap());
        file.time = commits[0].committer().when().seconds();
        file.msg = String::from(commits[0].summary().unwrap());
    }

    let diff = if start > end {
        start - end
    } else {
        end - start
    };
    let mid = (start + end) / 2;

    if diff == 1 || diff == 0 {
        let tree = trees[end].clone();
        match in_tree_handler(&Oid::from_str(&file.oid).unwrap(), tree, repo) {
            Some(_finally) => {
                file.commit = format!("{}", commits[end].id());
                file.author = String::from(commits[end].committer().name().unwrap());
                file.time = commits[end].committer().when().seconds();
                file.msg = String::from(commits[end].summary().unwrap());
            }
            None => {
                file.commit = format!("{}", commits[start].id());
                file.author = String::from(commits[start].committer().name().unwrap());
                file.time = commits[start].committer().when().seconds();
                file.msg = String::from(commits[start].summary().unwrap());
            }
        }
        return;
    }
    let tree = trees[mid].clone();
    match in_tree_handler(&Oid::from_str(&file.oid).unwrap(), tree, repo) {
        Some(_file1) => {
            let tree = trees[mid + 1].clone();
            match in_tree_handler(&Oid::from_str(&file.oid).unwrap(), tree, repo) {
                Some(_file2) => {
                    bsearch_deep(mid + 1, end, file, trees, commits, repo);
                }
                None => {
                    file.commit = format!("{}", commits[mid].id());
                    file.author = String::from(commits[mid].committer().name().unwrap());
                    file.time = commits[mid].committer().when().seconds();
                    file.msg = String::from(commits[mid].summary().unwrap());
                    return;
                }
            }
        }
        None => {
            let tree = trees[mid - 1].clone();
            match in_tree_handler(&Oid::from_str(&file.oid).unwrap(), tree, repo) {
                Some(_file2) => {
                    file.commit = format!("{}", commits[mid - 1].id());
                    file.author = String::from(commits[mid - 1].committer().name().unwrap());
                    file.time = commits[mid - 1].committer().when().seconds();
                    file.msg = String::from(commits[mid - 1].summary().unwrap());
                    return;
                }
                None => {
                    bsearch_deep(start, mid - 1, file, trees, commits, repo);
                }
            }
        }
    }
}

fn in_tree_handler(oid: &Oid, tree: Tree, repo: &Repository) -> Option<bool> {
    let mut truth = None;
    for sub_tree in tree
        .iter()
        .filter(|obj| obj.kind().unwrap().str() == "tree")
        .map(|tree| treeify(repo, tree))
    {
        match sub_tree.get_id(*oid) {
            Some(new_tree) => {
                if new_tree.id() == *oid {
                    truth = Some(true);
                    break;
                }
                if in_tree_handler(oid, treeify(repo, new_tree), repo).is_some() {
                    truth = Some(true);
                    break;
                }
            }
            None => {}
        }
    }
    truth
}

fn treeify<'a>(repo: &'a Repository, tree_to_be: TreeEntry) -> Tree<'a> {
    tree_to_be.to_object(repo).unwrap().peel_to_tree().unwrap()
}

fn _print_tree<'a>(tree: &git2::Tree, repo: &'a Repository) {
    println!("\nPrinting tree {:?} ", tree.id());
    for blob in tree.iter() {
        let blob_type = blob.kind().unwrap().str();
        println!("Blob_type: {:?}", blob_type);
        if blob_type == "tree" {
            let tree_cast = &blob.to_object(repo).unwrap().peel_to_tree().unwrap();
            println!(
                "Tree: {:?} | Oid: {:?} | parent {:?}",
                blob.name().unwrap(),
                blob.id(),
                tree.id()
            );
            _print_tree(tree_cast, repo);
        } else {
            println!("File: {:?} | Oid: {:?} ", blob.name().unwrap(), blob.id());
        }
    }
}

fn get_commits<'a>(oids: &'a Vec<Oid>, repo: &'a Repository) -> Vec<Commit<'a>> {
    oids.iter()
        .map(|oid| repo.find_commit(*oid).unwrap())
        .collect()
}

fn git_truth(file: String, parent: String, full_path: String) -> Result<GitCommit, ()> {
    let file_name = file.clone();
    let file_path = if parent.len() != 0 {
        format!("{}/{}", parent, file)
    } else {
        file
    };

    println!("Getting Latest Commit for file {:#?}", file_path);
    let output = Command::new("git")
        .arg("-C")
        .arg(full_path)
        .arg("log")
        .arg("--pretty=format:%H%n%cn%n%cD%n%s")
        .arg("-n")
        .arg("1")
        .arg(file_path)
        .output()
        .expect("git command failed to start");

    let stdout = output.stdout.as_ref();
    let hello = std::str::from_utf8(&stdout).unwrap();
    let hello = hello.split("\n").collect::<Vec<&str>>();

    if hello.len() != 4 {
        Err(())
    } else {
        Ok(GitCommit {
            file: file_name,
            commit: String::from(hello[0]),
            author: String::from(hello[1]),
            time: String::from(hello[2]),
            msg: String::from(hello[3]),
        })
    }
}

// fn main(){
//      let a = Oid::from_str("072d563d59c2efa35488517a87284a37f2127ced").unwrap();
//      let repo =open_repo("/Users/carlos/tensorflow".to_owned()).unwrap();
//      let _files = get_info(Some(a),repo).unwrap();
//      println!("FILES {:#?}",_files);
// }

#[cfg(test)]
mod tests {
    use super::*;

    //THIS FAILS BECAUSE GIT LOG ISN'T PULLING THE MOST UP TO DATE INFORMATION
    #[test]
    fn check_truth() {
        let path = String::from("/Users/carlos/tensorflow");
        let files = get_file_list(&path);
        let _truth_files = files.clone();
        let _truth_files: Vec<GitCommit> = _truth_files
            .iter()
            .map(|file| {
                let f2 = file.clone();
                let p2 = path.clone();
                git_truth(f2.name, String::from(""), p2).unwrap()
            }).collect();

        // let uncertain_files = get_info(None).unwrap();
        //
        // uncertain_files.iter().for_each(|file| {
        //     let _truth = truth_files.iter().find(|ref tfile| file.name==tfile.file ).unwrap();
        //     //assert_ne!(Oid::from_str(&truth.commit).unwrap(),file.commit,"FILE {} not equal to truth",file.name);
        // });
    }
}
