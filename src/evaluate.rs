use super::syntax::{Node};
use super::environment::{Environment};

pub trait Evaluate {
    fn evaluate(&self, environment: &mut Environment) -> Box<Node>;
}

impl Evaluate for Node {
    fn evaluate(&self, environment: &mut Environment) -> Box<Node> {
        match *self {
            Node::Number(v) => { Node::number(v) }
            Node::Boolean(v) => { Node::boolean(v) }
            Node::DoNothing => { Node::donothing() }
            Node::Add(ref l, ref r) => {
                Node::number(l.evaluate(environment).value() + r.evaluate(environment).value())
            }
            Node::Multiply(ref l, ref r) => {
                Node::number(l.evaluate(environment).value() * r.evaluate(environment).value())
            }
            Node::LT(ref l, ref r) => {
                Node::boolean(l.evaluate(environment).value() < r.evaluate(environment).value())
            }
            Node::EQ(ref l, ref r) => {
                Node::boolean(l.evaluate(environment).value() == r.evaluate(environment).value())
            }
            Node::GT(ref l, ref r) => {
                Node::lt(r.clone(), l.clone()).evaluate(environment)
            }
            Node::Variable(ref name) => {
                environment.get(&name)
            }
            Node::Assign(ref name, ref expr) => {
                let reduce = expr.evaluate(environment);
                environment.add(name, reduce.clone());
                Node::donothing()
            }
            Node::If(ref condition, ref consequence, ref alternative) => {
                if condition.evaluate(environment).condition() {
                    consequence.evaluate(environment)
                } else {
                    alternative.evaluate(environment)
                }
            }
            Node::Sequence(ref head, ref more) => {
                head.evaluate(environment);
                more.evaluate(environment);
                Node::donothing()
            }
            Node::While(ref cond, ref body) => {
                if cond.evaluate(environment).condition() {
                    body.evaluate(environment);
                    self.evaluate(environment)
                } else {
                    Node::donothing()
                }
            }
            Node::Pair(ref fst, ref snd) => {
                Node::pair(fst.clone(), snd.clone())
            }
            Node::Fst(ref pair) => {
                match **pair {
                    Node::Pair(ref l, ref _r) => l.clone(),
                    _ => panic!("Apply fst on non-pair type: {}", pair)
                }
            }
            Node::Snd(ref pair) => {
                match **pair {
                    Node::Pair(ref _l, ref r) => r.clone(),
                    _ => panic!("Apply snd on non-pair type: {}", pair)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_big_number() {
        let n = Node::number(3);
        let mut env = Environment::new();
        assert_eq!(3, n.evaluate(&mut env).value());
    }

    #[test]
    fn test_simple_big_variable() {
        let n = Node::variable("x");
        let mut env = Environment::new();
        env.add("x", Node::number(23));
        assert_eq!(23, n.evaluate(&mut env).value());
    }

    #[test]
    fn test_simple_big_arithmetic() {
        let n = Node::multiply(Node::number(14), Node::number(3));
        let mut env = Environment::new();
        assert_eq!(42, n.evaluate(&mut env).value());
    }

    #[test]
    fn test_simple_big_lessthan() {
        let n = Node::lt(Node::add(Node::variable("x"), Node::number(2)), Node::variable("y"));
        let mut env = Environment::new();
        env.add("x", Node::number(2));
        env.add("y", Node::number(5));
        assert!(n.evaluate(&mut env).condition());
    }

    #[test]
    fn test_simple_big_sequence() {
        let statement = Node::sequence(
            Node::assign("x", Node::add(Node::number(1), Node::number(1))),
            Node::assign("y", Node::add(Node::variable("x"), Node::number(3)))
        );
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(2, env.get("x").value());
        assert_eq!(5, env.get("y").value());
    }

    #[test]
    fn test_simple_big_while() {
        let statement = Node::while_node(
            Node::lt(Node::variable("x"), Node::number(5)),
            Node::assign("x", Node::multiply(Node::variable("x"), Node::number(3))),
        );
        let mut env = Environment::new();
        env.add("x", Node::number(1));
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(9, env.get("x").value());
    }
}
