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
                match *pair.evaluate(environment) {
                    Node::Pair(ref l, ref _r) => {
                        l.evaluate(environment).clone()
                    }
                    _ => panic!("Apply fst on non-pair type: {}", pair)
                }
            }
            Node::Snd(ref pair) => {
                match *pair.evaluate(environment) {
                    Node::Pair(ref _l, ref r) => {
                        r.evaluate(environment).clone()
                    }
                    _ => panic!("Apply snd on non-pair type: {}", pair)
                }
            }
            Node::Fun(ref _funname, ref _argname, ref _body) => {
                Node::closure(environment.clone(), Box::new(self.clone()))
            }
            Node::Closure(ref env, ref fun) => {
                Node::closure(env.clone(), fun.clone())
            }
            Node::Call(ref closure, ref arg) => {
                let arg = arg.evaluate(environment);
                match *closure.evaluate(environment) {
                    Node::Closure(ref mut env, ref fun) => {
                        if let Node::Fun(funname, argname, body) = *fun.clone() {
                            env.add(&funname, closure.clone());
                            if !argname.is_empty() {
                                env.add(&argname, arg.clone());
                            }
                            body.evaluate(env)
                        } else {
                            panic!("Closure not contain function: {}", fun)
                        }
                    }
                    _ => panic!("Call on non-closure type: {}", closure)
                }
            }
            _ => panic!("Non evaluate type found: {}", *self)
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

    #[test]
    fn test_simple_big_pair() {
        let statement = Node::sequence(
            Node::assign("y", Node::fst(Node::variable("p"))),
            Node::assign("z", Node::snd(Node::variable("p")))
        );
        let mut env = Environment::new();
        env.add("p", Node::pair(
            Node::add(Node::number(3), Node::number(4)),
            Node::multiply(Node::number(5), Node::number(6))
        ));
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(7, env.get("y").value());
        assert_eq!(30, env.get("z").value());
    }

    #[test]
    fn test_simple_big_function() {
        let statement = Node::assign("x", Node::call(
            Node::fun("const", "", Node::number(42)),
            Node::donothing()
            )
        );
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(42, env.get("x").value());
    }

    #[test]
    fn test_simple_big_function_var() {
        let add1 = Node::fun("add1", "x", Node::add(Node::variable("x"), Node::number(1)));
        let statement = Node::sequence(
            Node::assign("f", add1),
            Node::assign("result", Node::call(Node::variable("f"), Node::number(4)))
        );
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(5, env.get("result").value());
    }

    #[test]
    fn test_simple_big_function_env() {
        let x_add_y = Node::fun("add1", "y", Node::add(Node::variable("x"), Node::variable("y")));
        let statement = Node::sequence(
            Node::assign("x", Node::number(3)),
            Node::sequence(
                Node::assign("add3", x_add_y), // change x_add_y into y + 3
                Node::sequence(
                    Node::assign("x", Node::number(5)), // reassign x, and call function
                    Node::assign("result", Node::call(Node::variable("add3"), Node::number(4)))
                )
            )
        );
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(7, env.get("result").value());
    }

    #[test]
    fn test_simple_big_function_twoarg() {
        let x_add_y = Node::fun("addx", "x", Node::fun("addy", "y", Node::add(Node::variable("x"), Node::variable("y"))));
        let statement = Node::assign("result", Node::call(Node::call(x_add_y, Node::number(17)), Node::number(31)));
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(48, env.get("result").value());
    }
}
