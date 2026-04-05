use std::collections::VecDeque;
use crate::backend::NodeId;

const BUFFER_SIZE: usize = 15;

pub struct History {
    tape: VecDeque<NodeId>
}

impl History {
    pub fn new(init_state: NodeId) -> Self {
        let mut tape: VecDeque<NodeId> = VecDeque::with_capacity(BUFFER_SIZE);
        tape.push_front(init_state);
        Self { tape }
    }

    pub fn unwind(&mut self) {
        if self.tape.len() > 1 {
            self.tape.pop_back();
        }
    }

    pub fn enqueue(&mut self, state: NodeId) {
        if self.tape.len() % BUFFER_SIZE == 0 {
            self.tape.pop_front();
        }
        
        self.tape.push_back(state);
    }

    pub fn state(&self) -> NodeId {
        self.tape[self.tape.len() - 1]
    }
}
