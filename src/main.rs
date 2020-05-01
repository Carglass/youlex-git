use git2::Repository;
use git2::RepositoryInitOptions;

use std::result::Result;

struct Lex {
    repo: git2::Repository,
}

impl Lex {
    fn init_lex(name: &str) -> Lex {
        // options to ensure that we get an error when trying to init an existing repo
        let mut options = RepositoryInitOptions::new();
        options.no_reinit(true);
        options.bare(true);
        if let Ok(repo) = Repository::init_opts(format!("output/{}", name), &options) {
            Lex { repo }
        } else {
            panic!("Something went wrong with repo init");
        }
    }

    fn open(name: &str) -> Lex {
        if let Ok(repo) = Repository::open_bare(format!("output/{}", name)) {
            Lex { repo }
        } else {
            panic!("Something went wrong with repo open");
        }
    }

    fn create_content(&self, content: &str) -> Result<git2::Oid, git2::Error> {
        let as_bytes = content.as_bytes();
        self.repo.blob(as_bytes)
    }

    fn create_tree(&self, name: &str, children: git2::Oid) -> Result<git2::Oid, git2::Error> {
        let mut tree_builder = self.repo.treebuilder(None).unwrap();
        let config_content = self.create_content(name).unwrap();
        tree_builder.insert("0", config_content, 0o100644)?;
        // will need to take multiple children down the road
        // we use numbers to sort of represent an array in git
        tree_builder.insert("1", children, 0o100644)?;
        tree_builder.write()
    }
}

fn main() {
    println!("Hello, world!");

    let lex = Lex::open("alloitest2");
    // let is_bare = repo.is_bare().to_string();
    // println!("{}", is_bare);
    // if let Ok(id) = create_content(&repo, "test content") {
    //     if let Ok(blob) = repo.find_blob(id) {
    //         println!("{}", std::str::from_utf8(blob.content()).unwrap());
    //     }
    // }
    lex.create_tree(
        "and onother another ne",
        git2::Oid::from_str("08cf6101416f0ce0dda3c80e627f333854c4085c").unwrap(),
    )
    .unwrap();
}
