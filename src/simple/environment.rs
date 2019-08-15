use std::fmt::Display;
use std::fmt::Result;
use std::fmt::Formatter;

use super::syntax::Node;

use std::collections::HashMap;

#[derive(Debug,PartialEq,Clone)]
pub struct Environment {
    pub vars: HashMap<String, Box<Node>>
}

impl Environment {
    pub fn new() -> Environment {
        Environment{ vars: HashMap::new() }
    }

    pub fn add(&mut self, name: &str, node: Box<Node>) {
        self.vars.insert(name.to_string(), node);
    }

    pub fn get(&self, name: &str) -> Box<Node> {
        match self.vars.get(name) {
            Some(node) => node.clone(),
            None => panic!("Variable {} not found", name),
        }
    }

    pub fn prettyprint(&self, indent: usize) -> String {
        let prefix = " ".repeat(indent);
        let mut parts = Vec::new();
        for (key, val) in self.vars.iter() {
            parts.push(format!("{0}{1} = {2}\n",
                               prefix, key, val))
        };
        let text = parts.join("");
        format!("{0}{{\n{1}{0}}}", prefix, text)
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.prettyprint(0))
    }
}
