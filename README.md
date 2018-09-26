Package will retrieve latest commit for a given git repository specified by
environment variable GIT_PATH.

To run:

cd ./git_server
rustup default nightly
export GIT_PATH=// Wherever your gitpath is //
cargo run

To run in production:

cargo build --release
ROCKET_ENV=production ./target/release/git_server

Thanks to the teams at Rocket and Rust Libgit2.
