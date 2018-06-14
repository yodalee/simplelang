extern crate proglang;

use proglang::syntax::*;

pub fn vec_to_list(v: Vec<i64>) -> Box<Node> {
    v.iter().rev().fold(Node::donothing(), |cdr, car| Node::pair(Node::number(*car), cdr))
}

pub fn main() {
    let v = vec![1,2,3,4,5];
    println!("{}", vec_to_list(v));
}
