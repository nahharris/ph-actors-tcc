#[cfg(not(test))]
use crate::other_actor::OtherActor;

#[cfg(test)]
use crate::other_actor::mock::MockOtherActor as OtherActor;

pub struct App {
    dependency: OtherActor,
}

impl App {
    pub fn new(dependency: OtherActor) -> Self {
        Self { dependency }
    }
}

