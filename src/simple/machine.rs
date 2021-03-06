use super::syntax::Node;
use super::environment::Environment;
use super::evaluate::Evaluate;

pub struct Machine {
    pub environment: Environment,
    expression: Box<Node>,
}

impl Machine {
    pub fn new(expression: Box<Node>, environment: Environment) -> Machine {
        Machine{
            expression: expression,
            environment: environment,
        }
    }

    pub fn new_with_empty_env(expression: Box<Node>) -> Machine {
        Machine {
            expression: expression,
            environment: Environment::new(),
        }
    }

    pub fn run(&mut self) {
        self.expression.evaluate(&mut self.environment);
    }

    pub fn get_environment(&self) -> Environment {
        self.environment.clone()
    }
}
