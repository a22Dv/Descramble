use crate::data::{Args, Data, Frequency, State};
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
        todo!();
    }
}
