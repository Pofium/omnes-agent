use crate::util::{helper, other};

pub struct Engine {
    pub name: String,
}

pub trait Runner {
    fn run(&self) -> i32;
}

impl Runner for Engine {
    fn run(&self) -> i32 {
        helper()
    }
}

pub fn boot() -> i32 {
    let e = Engine {
        name: "x".to_string(),
    };
    e.run() + other()
}
