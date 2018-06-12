use super::syntax::{Node};
use super::environment::{Environment};

pub trait Reduce {
    fn reducible(&self) -> bool;
    fn reduce(&self, environment: &mut Environment) -> Box<Node>;
}

impl Reduce for Node {
    fn reducible(&self) -> bool {
        match *self {
            Node::Number(_) | Node::Boolean(_) => false,
            Node::DoNothing => false,
            Node::Closure(_, _) => false,
            Node::Pair(ref l, ref r) => l.reducible() || r.reducible(),
            _ => true,
        }
    }

    fn reduce(&self, environment: &mut Environment) -> Box<Node> {
        match *self {
            Node::Add(ref l, ref r) => {
                if l.reducible() {
                    Node::add(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::add(l.clone(), r.reduce(environment))
                } else {
                    Node::number(l.value() + r.value())
                }
            }
            Node::Subtract(ref l, ref r) => {
                if l.reducible() {
                    Node::subtract(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::subtract(l.clone(), r.reduce(environment))
                } else {
                    Node::number(l.value() - r.value())
                }
            }
            Node::Multiply(ref l, ref r) => {
                if l.reducible() {
                    Node::multiply(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::multiply(l.clone(), r.reduce(environment))
                } else {
                    Node::number(l.value() * r.value())
                }
            }
            Node::LT(ref l, ref r) => {
                if l.reducible() {
                    Node::lt(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::lt(l.clone(), r.reduce(environment))
                } else {
                    Node::boolean(l.value() < r.value())
                }
            }
            Node::EQ(ref l, ref r) => {
                if l.reducible() {
                    Node::eq(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::eq(l.clone(), r.reduce(environment))
                } else {
                    Node::boolean(l.value() == r.value())
                }
            }
            Node::GT(ref l, ref r) => { Node::lt(r.clone(), l.clone()) }
            Node::Variable(ref name) => {
                environment.get(&name)
            }
            Node::Assign(ref name, ref expr) => {
                if expr.reducible() {
                    Node::assign(name, expr.reduce(environment))
                } else {
                    environment.add(name, expr.clone());
                    Node::donothing()
                }
            }
            Node::If(ref condition, ref consequence, ref alternative) => {
                if condition.reducible() {
                    Node::if_cond_else(condition.reduce(environment), consequence.clone(), alternative.clone())
                } else {
                    if condition.condition() {
                        consequence.clone()
                    } else {
                        alternative.clone()
                    }
                }
            }
            Node::Sequence(ref head, ref more) => {
                match **head {
                    Node::DoNothing => more.clone(),
                    _ => Node::sequence(head.reduce(environment), more.clone()),
                }
            }
            Node::While(ref cond, ref body) => {
                Node::if_cond_else(
                    cond.clone(),
                    Node::sequence(body.clone(), Box::new(self.clone())),
                    Node::donothing()
                )
            }
            Node::Pair(ref l, ref r) => {
                if l.reducible() {
                    Node::pair(l.reduce(environment), r.clone())
                } else if r.reducible() {
                    Node::pair(l.clone(), r.reduce(environment))
                } else {
                    Node::pair(l.clone(), r.clone())
                }
            }
            Node::Fst(ref pair) => {
                if pair.reducible() {
                    Node::fst(pair.reduce(environment))
                } else {
                    match **pair {
                        Node::Pair(ref l, ref _r) => l.clone(),
                        _ => panic!("Apply fst on non-pair type: {}", pair)
                    }
                }
            }
            Node::Snd(ref pair) => {
                if pair.reducible() {
                    Node::snd(pair.reduce(environment))
                } else {
                    match **pair {
                        Node::Pair(ref _l, ref r) => r.clone(),
                        _ => panic!("Apply snd on non-pair type: {}", pair)
                    }
                }
            }
            Node::Fun(_, _, _) => {
                Node::closure(environment.clone(), Box::new(self.clone()))
            }
            _ => panic!("Non reducible type found: {}", *self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::machine::Machine;

    #[test]
    fn test_simple_small_number() {
        let n = Node::number(3);
        assert_eq!("3", format!("{}", n));
        assert!(!n.reducible());
    }

    #[test]
    fn test_simple_small_arithmetic() {
        let m = Node::add(
            Node::multiply(Node::number(1), Node::number(2)),
            Node::multiply(Node::number(3), Node::number(4)));
        assert!(m.reducible());
        let mut machine = Machine::new_with_empty_env(m);
        machine.run();
        assert!(!machine.get_expression().reducible());
        assert_eq!(14, machine.get_expression().value());
    }

    #[test]
    fn test_simple_small_lessthan() {
        let m = Node::lt(Node::number(5), Node::add(Node::number(2), Node::number(2)));
        let mut machine = Machine::new_with_empty_env(m);
        machine.run();
        assert!(!machine.get_expression().condition());
    }

    #[test]
    fn test_simple_small_variable() {
        let mut env = Environment::new();
        env.add("x", Node::number(3));
        env.add("y", Node::number(4));
        let mut machine = Machine::new(Node::add(Node::variable("x"), Node::variable("y")), env);
        machine.run();

        assert_eq!(7, machine.get_expression().value());
    }

    #[test]
    fn test_simple_small_statement() {
        let mut statement = Node::assign("x", Node::add(Node::variable("x"), Node::number(1)));
        let mut env = Environment::new();
        env.add("x", Node::number(2));

        assert!(statement.reducible());
        statement = statement.reduce(&mut env);
        println!("{0}; {1}", statement, env);
        statement = statement.reduce(&mut env);
        println!("{0}; {1}", statement, env);
        statement = statement.reduce(&mut env);
        println!("{0}; {1}", statement, env);
        assert!(!statement.reducible());
    }

    #[test]
    fn test_simple_small_true() {
        let mut env = Environment::new();
        env.add("x", Node::boolean(true));

        let mut machine = Machine::new(
            Node::if_cond_else(
                Node::variable("x"),
                Node::assign("y", Node::number(1)),
                Node::assign("y", Node::number(2))
            ), env
        );
        machine.run();
        assert_eq!(1, machine.environment.get("y").value());
    }

    #[test]
    #[should_panic]
    fn test_simple_small_false() {
        let mut env = Environment::new();
        env.add("x", Node::boolean(false));
        let mut machine = Machine::new(
            Node::if_cond_else(
                Node::variable("x"),
                Node::assign("y", Node::number(1)),
                Node::donothing()
            ), env
        );
        machine.run();
        assert!(machine.environment.get("y").condition()); // should blow up
    }

    #[test]
    fn test_simple_small_sequence() {
        let mut machine = Machine::new_with_empty_env(
            Node::sequence(
                Node::assign("x", Node::add(Node::number(1), Node::number(1))),
                Node::assign("y", Node::add(Node::variable("x"), Node::number(3))),
            )
        );
        machine.run();
        assert_eq!(2, machine.environment.get("x").value());
        assert_eq!(5, machine.environment.get("y").value());
    }

    #[test]
    fn test_simple_small_while() {
        let mut env = Environment::new();
        env.add("x", Node::number(1));
        let mut machine = Machine::new(
            Node::while_node(
                Node::lt(Node::variable("x"), Node::number(5)),
                Node::assign("x", Node::multiply(Node::variable("x"), Node::number(3)))
            ), env
        );

        machine.run();
        assert_eq!(9, machine.environment.get("x").value());
    }

    #[test]
    fn test_simple_small_pair() {
        let mut env = Environment::new();
        env.add("p", Node::pair(
            Node::add(Node::number(3), Node::number(4)),
            Node::multiply(Node::number(5), Node::number(6))
        ));
        let mut machine = Machine::new(
            Node::sequence(
                Node::assign("y", Node::fst(Node::variable("p"))),
                Node::assign("z", Node::snd(Node::variable("p")))
            ), env
        );

        machine.run();
        assert_eq!(7, machine.environment.get("y").value());
        assert_eq!(30, machine.environment.get("z").value());
    }
}
