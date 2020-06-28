//! # Youlex-Git
//! 
//! This crate is a library designed to help store structured text (arborescent table of contents) into a Git repository
//! This provides some advantages over using a database: 
//! - It is versioned by design.
//! - It allows users to collaborate and merge their work, using beloved git capabilities.
//! - Decentralization opens the door for extended offline capabilities.
//! 
//! A lot of fields can potentially use versioned trees for collaborative work, but it is mostly only accessible to developpers.
//! Youlex intends to bring these capabilities to the field of legislation, but this crate will be designed to provide a generic API.
//! 
//! ## Data Model
//! 
//! As the main target is web applications, we will work on storing json files into git. 

use git2::Repository;
use git2::RepositoryInitOptions;
use serde::{Deserialize, Serialize};

use std::result::Result;

fn connect(url: &str) {}

mod schema {
    struct Schema
}



#[derive(Serialize, Deserialize)]
struct Node {
    title: String,
    // should we try to get a better type for that? may need a lib to generate them anyway
    id: String,
    description: String,
    children: Vec<TreeItem>,
}

impl Node {
    fn walk<C, D, E>(&self, self_callback: C, leaf_callback: D) -> E
    where
        C: Fn(&Node, Vec<E>) -> E,
        D: Fn(&Leaf) -> E,
    {
        // this walk will be run in post-order
        // create a vec to store the results of the children callbacks
        let mut children_res : Vec<E> = vec![];
        // get an iterator over the children
        let iter = self.children.iter();
        // iterate over children
        for child in iter {
            match child {
                // walk on the child node, push the result into our vec
                TreeItem::Node(node) => children_res.push(node.walk(self_callback, leaf_callback)),
                // execute leaf callback on a leaf, push the result into our vec
                TreeItem::Leaf(leaf) => children_res.push(leaf_callback(&leaf)),
            }
        }
        // execute callback on self, plus the array of children result, return its result
        self_callback(&self, children_res)
    }
}

#[derive(Serialize, Deserialize)]
struct Leaf {
    title: String,
    id: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
enum TreeItem {
    Node(Node),
    Leaf(Leaf),
}

struct Lex {
    repo: git2::Repository,
    contents: Option<Node>,
}

impl Lex {
    fn init_lex(name: &str) -> Lex {
        // options to ensure that we get an error when trying to init an existing repo
        let mut options = RepositoryInitOptions::new();
        options.no_reinit(true);
        options.bare(true);
        if let Ok(repo) = Repository::init_opts(format!("output/{}", name), &options) {
            Lex {
                repo,
                contents: None,
            }
        } else {
            panic!("Something went wrong with repo init");
        }
    }

    fn open(name: &str) -> Lex {
        if let Ok(repo) = Repository::open_bare(format!("output/{}", name)) {
            // TODO clearly there is something to do here to parse the actual current content in the HEAD? 
            Lex { repo, contents: None }
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

    fn push_contents(&self, node: Node) {
        self.contents = Some(node);
    }

    fn save(&self) {
        // TODO to consider taking here external contents, diff them with lex contents which would be head, and then save only what needs to be saved? 
        if let Some(root) = &self.contents {
            // we need to store all contents in the tree
            // to do that we walk over the contents, passing a store in git callback
            let node_callback = |node: Node, info_array: Vec<(git2::Oid, i32)>| -> (git2::Oid, i32) {
                let mut tree_builder = self.repo.treebuilder(None).unwrap();
                // add the node info in a json file

                // TODO here save the node info

                // loop over children save info array to add them
                let count = 1;
                for info in info_array.iter() {
                    tree_builder.insert(count.to_string(), info.0, info.1);
                }
                let tree_oid = tree_builder.write().unwrap();
                return (tree_oid, 0o040000);
            };
            // TODO create the leaf_callback

            // TODO return the root oid, or rather write index and commit? 
        } else {
            panic!("No content to save")
        }
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
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}