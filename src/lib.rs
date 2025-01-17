mod program;

use program::Program;

pub fn run() {
    let program = Program::new().unwrap();
    program.run().unwrap();
}
