use git2::Repository;
use git2::RepositoryInitOptions;
use serde::{Deserialize, Serialize};

use std::result::Result;

#[derive(Serialize, Deserialize)]
struct Article {
    title: String,
    alineas: Vec<String>,
}

impl Article {
    fn new(json_as_string: &str) -> Article {
        serde_json::from_str(json_as_string).unwrap()
    }
}

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

    fn convert_serde_json_into_tree(&self, article: Article) -> Result<git2::Oid, git2::Error> {
        let mut alineas_tree_builder = self.repo.treebuilder(None).unwrap();
        alineas_tree_builder.insert(
            "1",
            self.create_content(article.alineas[0].as_str())?,
            0o100644,
        )?;
        alineas_tree_builder.insert(
            "2",
            self.create_content(article.alineas[1].as_str())?,
            0o100644,
        )?;
        let alineas_oid = alineas_tree_builder.write().unwrap();
        let mut tree_builder = self.repo.treebuilder(None).unwrap();
        tree_builder.insert(
            "0",
            self.create_content(article.title.as_str()).unwrap(),
            0o100644,
        )?;
        tree_builder.insert("alineas", alineas_oid, 0o040000)?;
        tree_builder.write()
    }

    fn convert_tree_into_serde_json(&self, tree_oid: git2::Oid) -> Result<Article, git2::Error> {
        let tree = self.repo.find_tree(tree_oid).unwrap();
        let mut title: String = "title to change".to_owned();
        let mut alineas: Vec<String> = vec![];
        tree.walk(
            git2::TreeWalkMode::PreOrder,
            |root: &str, entry: &git2::TreeEntry| {
                match root {
                    "" => {
                        if let Some(git2::ObjectType::Blob) = entry.kind() {
                            title = std::str::from_utf8(
                                entry
                                    .to_object(&self.repo)
                                    .unwrap()
                                    .as_blob()
                                    .unwrap()
                                    .content(),
                            )
                            .unwrap()
                            .to_owned()
                        }
                    }
                    "alineas/" => alineas.push(
                        std::str::from_utf8(
                            entry
                                .to_object(&self.repo)
                                .unwrap()
                                .as_blob()
                                .unwrap()
                                .content(),
                        )
                        .unwrap()
                        .to_owned(),
                    ),
                    _ => println!("nothing happens"),
                }
                println!("{}", root);
                git2::TreeWalkResult::Ok
            },
        )
        .unwrap();
        Ok(Article { title, alineas })
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

    let data = r#"
        {
            "title": "Article",
            "alineas": [
                "Hello",
                "World"
            ]
        }"#;
    let article = Article::new(data);
    let tree_oid = lex.convert_serde_json_into_tree(article).unwrap();
    let article_again = lex.convert_tree_into_serde_json(tree_oid).unwrap();
    println!("{}", article_again.title);
    println!("{}", article_again.alineas[0]);
    println!("{}", article_again.alineas[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn init_a_lex_works() -> Result<(), ()> {
        // clean anything that existed previously
        // technically we should check that the error if any from remove_dir_all is not found which just means the test probably never ran before
        fs::remove_dir_all("output/test_lex");
        Lex::init_lex("test_lex");
        match fs::read_dir("output/test_lex/objects") {
            Ok(something) => Ok(()),
            Err(error) => Err(()),
        }
    }
}
