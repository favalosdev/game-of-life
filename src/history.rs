use std::collections::VecDeque;
use crate::backend::NodeId;

pub struct History {
    tape: VecDeque<NodeId>,
    pointer: usize
}

impl History {
    pub fn new(capacity: usize, init_state: NodeId) -> Self {
        let mut tape: VecDeque<NodeId> = VecDeque::with_capacity(capacity);
        tape.push_back(init_state);

        Self {
            tape,
            pointer: 0
        }
    }

    pub fn unwind(&mut self) {
        if self.pointer > 0 {
            self.pointer -= 1;
        }
    }

    pub fn rewind(&mut self) {
        if self.pointer + 1 < self.tape.len() {
            self.pointer += 1;
        }
    }

    pub fn enqueue(&mut self, state: NodeId) {
        // Flush
        if (self.tape.len() - 1) - self.pointer >= 1 {
            self.tape.truncate(self.pointer + 1);
        }

        let n = self.tape.len();

        if n > 0 && n % self.tape.capacity() == 0 {
            self.tape.pop_front();
            self.unwind();
        }

        self.tape.push_back(state);
        self.rewind();
    }

    pub fn state(&self) -> NodeId {
        self.tape[self.pointer]
    }
}
