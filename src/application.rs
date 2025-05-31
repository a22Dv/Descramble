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
        dbg!(&solutions);
    }
}
