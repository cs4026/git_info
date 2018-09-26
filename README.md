Package will retrieve latest commit for a given git repository specified by
environment variable GIT_PATH.

To run:

cd ./git_server <br/>
rustup default nightly <br/>
export GIT_PATH=// Wherever your gitpath is // <br/>
cargo run

To run in production:

cargo build --release <br/>
ROCKET_ENV=production ./target/release/git_server <br/>

Thanks to the teams at Rocket and Rust Libgit2.

API:

GET /repo/<username>/<repo_name>/<tree_id>

username = username of human
repo_name = name of repo owned by human
// IMPORTANT //
tree_id = The Object ID of a tree ( tree = directory )<br/>
If you only want to get the top level information you must use "VOID" as tree_id <br/>

ex.

http://localhost:9999/repo/carlos/git_server/VOID
-or-
http://localhost:9999/repo/carlos/git_server/3aaed8b8c62156c37b24bc9a9d4bd916225f9dec
