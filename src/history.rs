use std::collections::VecDeque;
use golback::universe::NodeId;

const CAPACITY: usize = 5000;

pub struct History {
    tape: VecDeque<NodeId>,
    pointer: usize
}

impl History {
    pub fn new(init_state: NodeId) -> Self {
        let mut tape: VecDeque<NodeId> = VecDeque::with_capacity(CAPACITY);
        tape.push_back(init_state);

        Self {
            tape,
            pointer: 0
        }
    }

    pub fn unwind(&mut self) {
        self.pointer -= 1;
    }

    pub fn rewind(&mut self) {
        self.pointer += 1;
    }

    pub fn can_rewind(&self) -> bool {
        (self.tape.len() - 1) - self.pointer >= 1 
    }

    pub fn can_unwind(&self) -> bool {
        self.pointer > 0
    }

    pub fn enqueue(&mut self, state: NodeId) {
        // Flush
        if self.can_rewind() {
            self.tape.truncate(self.pointer + 1);
        }

        let n = self.tape.len();

        if n > 0 && n % CAPACITY == 0 {
            self.tape.pop_front();
            self.pointer -= 1;
        }

        self.tape.push_back(state);
        self.pointer += 1;
    }

    pub fn state(&self) -> NodeId {
        self.tape[self.pointer]
    }
}
