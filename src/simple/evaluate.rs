use super::syntax::{Node};
use super::environment::{Environment};
use std::collections::HashSet;

pub trait Evaluate {
    fn evaluate(&self, environment: &mut Environment) -> Box<Node>;
}

fn get_free_vars_helper(node: &Box<Node>, varlist: &mut HashSet<String>, free_vars: &mut HashSet<String>) {
    match **node {
        Node::IsDoNothing(ref node) | Node::Fst(ref node) | Node::Snd(ref node) => {
            get_free_vars_helper(node, varlist, free_vars);
        }
        Node::Add(ref l, ref r) | Node::Subtract(ref l, ref r) |
            Node::Multiply(ref l, ref r) | Node::LT(ref l, ref r) |
            Node::EQ(ref l, ref r) | Node::GT(ref l, ref r) |
            Node::Sequence(ref l, ref r) | Node::While(ref l, ref r) |
            Node::Pair(ref l, ref r) => {
                get_free_vars_helper(l, varlist, free_vars);
                get_free_vars_helper(r, varlist, free_vars);
        }
        Node::Variable(ref name) => { 
            if !varlist.contains(name) {
                free_vars.insert(name.clone());
            }
        }
        Node::Assign(ref name, ref expr) => {
            get_free_vars_helper(expr, varlist, free_vars);
            varlist.insert(name.clone());
        }
        Node::If(ref condition, ref consequence, ref alternative) => {
            get_free_vars_helper(condition, varlist, free_vars);
            get_free_vars_helper(consequence, varlist, free_vars);
            get_free_vars_helper(alternative, varlist, free_vars);
        }
        Node::Fun(ref funname, ref argname, ref body) => {
            varlist.insert(funname.clone());
            varlist.insert(argname.clone());
            get_free_vars_helper(body, varlist, free_vars);
        }
        Node::Closure(ref _env, ref fun) => {
            get_free_vars_helper(fun, varlist, free_vars);
        }
        // Number, Boolean, DoNothing
        _ => (),
    }
}

fn get_free_vars(node: &Box<Node>) -> HashSet<String> {
    let mut vars: HashSet<String> = HashSet::new();
    let mut free_vars: HashSet<String> = HashSet::new();
    get_free_vars_helper(node, &mut vars, &mut free_vars);
    free_vars
}

impl Evaluate for Node {
    fn evaluate(&self, env: &mut Environment) -> Box<Node> {
        println!("evaluate {} with environment \n{}\n", self, env.prettyprint(0));
        match *self {
            Node::Number(v) => { Node::number(v) }
            Node::Boolean(v) => { Node::boolean(v) }
            Node::DoNothing => { Node::donothing() }
            Node::IsDoNothing(ref node) => {
                let node = node.evaluate(env);
                match *node {
                    Node::DoNothing => Node::boolean(true),
                    _ => Node::boolean(false),
                }
            }
            Node::Add(ref l, ref r) => {
                Node::number(l.evaluate(env).value() + r.evaluate(env).value())
            }
            Node::Subtract(ref l, ref r) => {
                Node::number(l.evaluate(env).value() - r.evaluate(env).value())
            }
            Node::Multiply(ref l, ref r) => {
                Node::number(l.evaluate(env).value() * r.evaluate(env).value())
            }
            Node::LT(ref l, ref r) => {
                Node::boolean(l.evaluate(env).value() < r.evaluate(env).value())
            }
            Node::EQ(ref l, ref r) => {
                Node::boolean(l.evaluate(env).value() == r.evaluate(env).value())
            }
            Node::GT(ref l, ref r) => {
                Node::lt(r.clone(), l.clone()).evaluate(env)
            }
            Node::Variable(ref name) => { env.get(&name) }
            Node::Assign(ref name, ref expr) => {
                let reduce = expr.evaluate(env);
                env.add(name, reduce.clone());
                Node::donothing()
            }
            Node::If(ref condition, ref consequence, ref alternative) => {
                if condition.evaluate(env).condition() {
                    consequence.evaluate(env)
                } else {
                    alternative.evaluate(env)
                }
            }
            Node::Sequence(ref head, ref more) => {
                head.evaluate(env);
                more.evaluate(env);
                Node::donothing()
            }
            Node::While(ref cond, ref body) => {
                if cond.evaluate(env).condition() {
                    body.evaluate(env);
                    self.evaluate(env)
                } else {
                    Node::donothing()
                }
            }
            Node::Pair(ref fst, ref snd) => {
                Node::pair(fst.evaluate(env).clone(), snd.evaluate(env).clone())
            }
            Node::Fst(ref pair) => {
                match *pair.evaluate(env) {
                    Node::Pair(ref l, ref _r) => {
                        l.evaluate(env).clone()
                    }
                    _ => panic!("Apply fst on non-pair type: {}", pair)
                }
            }
            Node::Snd(ref pair) => {
                match *pair.evaluate(env) {
                    Node::Pair(ref _l, ref r) => {
                        r.evaluate(env).clone()
                    }
                    _ => panic!("Apply snd on non-pair type: {}", pair)
                }
            }
            Node::Fun(ref _funname, ref _argname, ref _body) => {
                Node::closure(env.clone(), Box::new(self.clone()))
            }
            Node::Closure(ref env, ref fun) => {
                Node::closure(env.clone(), fun.clone())
            }
            Node::Call(ref closure, ref arg) => {
                let arg = arg.evaluate(env);
                let clsr = closure.evaluate(env);
                match *clsr {
                    Node::Closure(ref env, ref fun) => {
                        if let Node::Fun(funname, argname, body) = *fun.clone() {
                            let freevars = get_free_vars(&fun);
                            let mut newenv = Environment::new();
                            for var in freevars {
                                newenv.add(&var, env.get(&var));
                            }
                            newenv.add(&funname, clsr.clone());
                            if !argname.is_empty() {
                                newenv.add(&argname, arg.clone());
                            }
                            body.evaluate(&mut newenv)
                        } else {
                            panic!("Closure not contain function: {}", fun)
                        }
                    }
                    _ => panic!("Call on non-closure type: {:?}", closure)
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

    #[test]
    fn test_simple_big_function_recursive() {
        let factor = Node::fun("factor", "x", Node::if_cond_else(
                Node::gt(Node::variable("x"), Node::number(1)),
                Node::multiply(Node::variable("x"),
                               Node::call(Node::variable("factor"), Node::subtract(Node::variable("x"), Node::number(1)))),
                Node::number(1)));
        let statement = Node::sequence(
            Node::assign("entry", factor),
            Node::assign("result", Node::call(Node::variable("entry"), Node::number(10)))
        );
        let mut env = Environment::new();
        println!("{}", statement.evaluate(&mut env));
        assert_eq!(3628800, env.get("result").value());
    }

    #[test]
    fn test_get_free_vars() {
        let x_add_y = Node::fun("addx", "x", Node::fun("addy", "y", Node::add(Node::variable("x"), Node::variable("y"))));
        let freevars = get_free_vars(&x_add_y).iter().cloned().collect::<Vec<String>>();
        assert!(freevars.is_empty());

        let add_y = Node::fun("addy", "x", Node::add(Node::variable("x"), Node::variable("y")));
        let freevars = get_free_vars(&add_y).iter().cloned().collect::<Vec<String>>();
        assert!(!freevars.is_empty());
        assert_eq!("y", &freevars[0]);
    }
}
