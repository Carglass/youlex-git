use git2::Repository;
use git2::RepositoryInitOptions;

use std::result::Result;

fn init_lex(name: &str) -> Result<Repository, git2::Error> {
    // options to ensure that we get an error when trying to init an existing repo
    let mut options = RepositoryInitOptions::new();
    options.no_reinit(true);
    options.bare(true);
    Repository::init_opts(format!("output/{}", name), &options)
}

fn open_lex(name: &str) -> std::result::Result<Repository, git2::Error> {
    Repository::open_bare(format!("output/{}", name))
}

fn main() {
    println!("Hello, world!");

    let repo = match open_lex("alloitest2") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };
    let is_bare = repo.is_bare().to_string();
    println!("{}", is_bare);
    
}
