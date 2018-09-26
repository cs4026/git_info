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
