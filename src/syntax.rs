use super::environment::Environment;

use std::fmt::Display;
use std::fmt::Result;
use std::fmt::Formatter;

#[derive(Debug,PartialEq,Clone)]
pub enum Node {
    Number(i64),
    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
    Boolean(bool),
    LT(Box<Node>, Box<Node>),
    EQ(Box<Node>, Box<Node>),
    GT(Box<Node>, Box<Node>),
    Variable(String),
    DoNothing,
    Assign(String, Box<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
    Sequence(Box<Node>, Box<Node>),
    While(Box<Node>, Box<Node>),
    Pair(Box<Node>, Box<Node>),
    Fst(Box<Node>),
    Snd(Box<Node>),
    Fun(String, String, Box<Node>),
    Closure(Environment, Box<Node>),
    Call(Box<Node>, Box<Node>),
}

impl Node {
    pub fn number(value: i64) -> Box<Node> { Box::new(Node::Number(value)) }
    pub fn add(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::Add(left, right)) }
    pub fn subtract(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::Subtract(left, right)) }
    pub fn multiply(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::Multiply(left, right)) }
    pub fn boolean(value: bool) -> Box<Node> { Box::new(Node::Boolean(value)) }
    pub fn lt(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::LT(left, right)) }
    pub fn eq(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::EQ(left, right)) }
    pub fn gt(left: Box<Node>, right: Box<Node>) -> Box<Node> { Box::new(Node::GT(left, right)) }
    pub fn variable(name: &str) -> Box<Node> { Box::new(Node::Variable(name.to_string())) }
    pub fn donothing() -> Box<Node> { Box::new(Node::DoNothing) }
    pub fn assign(name: &str, expr: Box<Node>) -> Box<Node> { Box::new(Node::Assign(name.to_string(), expr)) }
    pub fn if_cond_else(condition: Box<Node>, consequence: Box<Node>, alternative: Box<Node>) -> Box<Node> {
        Box::new(Node::If(condition, consequence, alternative))
    }
    pub fn sequence(head: Box<Node>, more: Box<Node>) -> Box<Node> { Box::new(Node::Sequence(head, more)) }
    pub fn while_node(cond: Box<Node>, body: Box<Node>) -> Box<Node> { Box::new(Node::While(cond, body)) }
    pub fn pair(fst: Box<Node>, snd: Box<Node>) -> Box<Node> { Box::new(Node::Pair(fst, snd)) }
    pub fn fst(pair: Box<Node>) -> Box<Node> { Box::new(Node::Fst(pair)) }
    pub fn snd(pair: Box<Node>) -> Box<Node> { Box::new(Node::Snd(pair)) }
    pub fn fun(funname: &str, argname: &str, body: Box<Node>) -> Box<Node> {
        Box::new(Node::Fun(funname.to_string(), argname.to_string(), body))
    }
    pub fn closure(env: Environment, fun: Box<Node>) -> Box<Node> { Box::new(Node::Closure(env, fun)) }
    pub fn call(closure: Box<Node>, arg: Box<Node>) -> Box<Node> { Box::new(Node::Call(closure, arg)) }

    pub fn value(&self) -> i64 {
        match *self {
            Node::Number(value) => { value },
            _ => panic!("Type has no value: {}", *self)
        }
    }

    pub fn condition(&self) -> bool {
        match *self {
            Node::Boolean(b) => { b },
            _ => panic!("Type cannot eval to boolean {}", *self)
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Node::Number(value) => write!(f, "{}", value),
            Node::Add(ref l, ref r) => write!(f, "{0} + {1}", l, r),
            Node::Subtract(ref l, ref r) => write!(f, "{0} - {1}", l, r),
            Node::Multiply(ref l, ref r) => write!(f, "{0} * {1}", l, r),
            Node::Boolean(value) => write!(f, "{}", value),
            Node::LT(ref l, ref r) => write!(f, "{0} < {1}", l, r),
            Node::EQ(ref l, ref r) => write!(f, "{0} = {1}", l, r),
            Node::GT(ref l, ref r) => write!(f, "{0} > {1}", l, r),
            Node::Variable(ref name) => write!(f, "{}", name),
            Node::DoNothing => write!(f, "do-nothing"),
            Node::Assign(ref name, ref expr) => write!(f, "{0} = {1}", name, expr),
            Node::If(ref condition, ref consequence, ref alternative) => write!(f, "if ({0}) {1} else {2}", condition, consequence, alternative),
            Node::Sequence(ref head, ref more) => write!(f, "{0}; {1}", head, more),
            Node::While(ref cond, ref body) => write!(f, "while ({0}) {1}", cond, body),
            Node::Pair(ref fst, ref snd) => write!(f, "pair ({0}, {1})", fst, snd),
            Node::Fst(ref pair) => write!(f, "fst ({0})", pair),
            Node::Snd(ref pair) => write!(f, "snd ({0})", pair),
            Node::Fun(ref fname, ref argname, ref body) => write!(f, "function {0} ({1}) {2}", fname, argname, body),
            Node::Closure(ref env, ref fun) => write!(f, "closure with {0}", fun),
            Node::Call(ref closure, ref arg) => write!(f, "call {0} with {1}", closure, arg),
        }
    }
}
