extern crate proglang;
extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;

use proglang::simple::syntax::{Node};
use proglang::simple::machine::{Machine};

use pest::Parser;
use pest::iterators::{Pair};
use pest::prec_climber::{Assoc, PrecClimber, Operator};

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
    let ast = build_stats(pair);
    let mut machine = Machine::new_with_empty_env(ast);
    machine.run();
    println!("{}", machine.get_environment().get("result"));
}

fn build_stats(pair: Pair<Rule>) -> Box<Node> {
    let inner = pair.into_inner();
    let nodes : Vec<_> = inner.into_iter().map(|pair| build_stat(pair)).collect();
    if nodes.is_empty() {
        Node::donothing()
    } else {
        let first = nodes[0].clone();
        nodes.iter().skip(1).fold(
            first, |acc, node| Node::sequence(acc, node.clone()))
    }
}

fn build_stat(pair: Pair<Rule>) -> Box<Node> {
    match pair.as_rule() {
        Rule::stat_assign => build_assign(pair),
        Rule::stat_if => build_if(pair),
        Rule::stat_while => build_while(pair),
        Rule::stat_func => build_func(pair),
        Rule::expr => climb(pair),
        _ => unreachable!(),
    }
}

fn build_assign(pair: Pair<Rule>) -> Box<Node> {
    let mut inner = pair.into_inner();
    let lhs = inner.next().unwrap().as_span().as_str();
    let rhs = climb(inner.next().unwrap());
    Node::assign(lhs, rhs)
}

fn build_if(pair: Pair<Rule>) -> Box<Node> {
    let mut inner = pair.into_inner();
    let cond = climb(inner.next().unwrap());
    let then = build_stats(inner.next().unwrap());
    match inner.next() {
        Some(stmt) => Node::if_cond_else(cond, then, build_stats(stmt)),
        None => Node::if_cond_else(cond, then, Node::donothing())
    }
}

fn build_while(pair: Pair<Rule>) -> Box<Node> {
    let mut inner = pair.into_inner();
    let cond = climb(inner.next().unwrap());
    let stmt = build_stats(inner.next().unwrap());
    Node::while_node(cond, stmt)
}

fn build_func(pair: Pair<Rule>) -> Box<Node> {
    let mut inner = pair.into_inner();
    let funcname = inner.next().unwrap().as_span().as_str();
    let mut next = inner.next().unwrap();
    let mut argname = "";
    if next.as_rule() == Rule::variable {
        argname = next.as_span().as_str();
        next = inner.next().unwrap();
    }
    let body = build_stats(next);
    Node::assign(funcname, Node::fun(funcname, argname, body))
}

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = build_precedence_climber();
}

fn build_precedence_climber() -> PrecClimber<Rule> {
    PrecClimber::new(vec![
        Operator::new(Rule::op_mul, Assoc::Left),
        Operator::new(Rule::op_add, Assoc::Left) |
        Operator::new(Rule::op_sub, Assoc::Left),
        Operator::new(Rule::op_lt,  Assoc::Left) |
        Operator::new(Rule::op_gt,  Assoc::Left),
        Operator::new(Rule::op_eq,  Assoc::Left),
    ])
}

fn infix_rule(lhs: Box<Node>, op: Pair<Rule>, rhs: Box<Node>) -> Box<Node> {
    match op.as_rule() {
        Rule::op_add => Node::add(lhs, rhs),
        Rule::op_sub => Node::subtract(lhs, rhs),
        Rule::op_mul => Node::multiply(lhs, rhs),
        Rule::op_lt  => Node::lt(lhs, rhs),
        Rule::op_gt  => Node::gt(lhs, rhs),
        Rule::op_eq  => Node::eq(lhs, rhs),
        _ => unreachable!(),
    }
}

fn climb(pair: Pair<Rule>) -> Box<Node> {
    PREC_CLIMBER.climb(pair.into_inner(), build_factor, infix_rule)
}

fn build_factor(pair: Pair<Rule>) -> Box<Node> {
    match pair.as_rule() {
        Rule::variable => Node::variable(pair.as_span().as_str()),
        Rule::number => Node::number(pair.as_span().as_str().parse::<i64>().unwrap()),
        Rule::expr => climb(pair),
        Rule::call => build_call(pair),
        _ => unreachable!(),
    }
}

fn build_call(pair: Pair<Rule>) -> Box<Node> {
    let mut inner = pair.into_inner();
    let funcname = inner.next().unwrap().as_span().as_str();
    match funcname.as_ref() {
        "pair" => Node::pair(climb(inner.next().unwrap()), climb(inner.next().unwrap())),
        "fst"  => Node::fst(climb(inner.next().unwrap())),
        "snd"  => Node::snd(climb(inner.next().unwrap())),
        &_     => {
            let mut args : Vec<_> = inner.map(|pair| climb(pair)).collect();
            if args.is_empty() {
                args.push(Node::donothing());
            }
            args.iter().fold(Node::variable(funcname), |acc, arg| Node::call(acc, arg.clone()))
        }
    }
}

fn iterate_rules(pair: Pair<Rule>, indent: usize) {
    let span = pair.clone().as_span();
    let indentstr = "| ".repeat(indent);
    println!("{}Rule: {:?}, span '{}'",
             indentstr, pair.as_rule(), span.as_str().replace("\n", " "));
    for innerpair in pair.into_inner() {
        iterate_rules(innerpair, indent+2);
    }
}
