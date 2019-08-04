extern crate proglang;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use proglang::simple::syntax::{Node};
use proglang::simple::machine::{Machine};
use pest::Parser;
use pest::iterators::{Pair};
use std::env;
use std::process;
use std::fs::File;
use std::io::prelude::*;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("simple.pest");

#[derive(Parser)]
#[grammar = "simple.pest"]
struct SimpleParser;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ./simple-parser <source file>");
        process::exit(1);
    }
    for arg in env::args().skip(1) {
        let mut f = File::open(&arg).expect(&format!("file {} not found", arg));
        let mut content = String::new();
        f.read_to_string(&mut content).expect(&format!("Error in reading file {}", arg));
        parse_simple(&content);
    }
}

fn parse_simple(content: &str) {
    let pair = SimpleParser::parse(Rule::simple, content)
        .unwrap_or_else(|e| panic!("{}", e))
        .next().unwrap();
    iterate_rules(pair.clone(), 0);
    let ast = build_stats(&pair);
    let mut machine = Machine::new_with_empty_env(ast);
    machine.run();
}

fn build_stats(pair: &Pair<Rule>) -> Box<Node> {
    Node::donothing()
}

fn iterate_rules(pair: Pair<Rule>, indent: usize) {
    let span = pair.clone().into_span();
    let indentstr = "| ".repeat(indent);
    println!("{}Rule: {:?}, span '{}'",
             indentstr, pair.as_rule(), span.as_str().replace("\n", " "));
    for innerpair in pair.into_inner() {
        iterate_rules(innerpair, indent+2);
    }
}
