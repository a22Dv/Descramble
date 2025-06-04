use crate::{data::{State}, algorithm::Solutions};

pub struct Application {
    application_state: State,
}

impl Application {
    pub fn new(application_state: State) -> Self {
        Application {
            application_state: application_state
        }
    }
    pub fn start(&self) {
        let solutions: Solutions = Solutions::from(&self.application_state);
        let parsed_solution: Vec<(String, f64)> = solutions.parse(&self.application_state);
        for solution in parsed_solution.iter() {
            println!("{} - {}", solution.0, solution.1)
        }
    }
}
