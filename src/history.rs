use std::collections::VecDeque;
use crate::backend::NodeId;

const BUFFER_SIZE: usize = 1000;

pub struct History {
    tape: VecDeque<NodeId>,
    pointer: usize
}

impl History {
    pub fn new() -> Self {
        Self {
            tape: VecDeque::with_capacity(BUFFER_SIZE),
            pointer: 0
        }
    }

    pub fn forward(&mut self) {
        if self.pointer < self.tape.len() - 1 {
            self.pointer += 1;
        }
    }

    pub fn unwind(&mut self) {
        if self.pointer > 0 {
            self.pointer -= 1;
        }
    }

    pub fn push(&mut self, state: NodeId) {
        if ((self.tape.len() + 1) % BUFFER_SIZE) == 0 {
            self.tape.pop_front();
            self.pointer -= 1;
        }
        
        self.tape.push_back(state);
    }

    pub fn flush(&mut self) {
        self.tape.truncate(self.pointer + 1);
    }

    pub fn state(&self) -> NodeId {
        self.tape[self.pointer]
    }
}
