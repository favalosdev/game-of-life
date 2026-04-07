use std::collections::VecDeque;
use golback::universe::NodeId;

pub struct History {
    tape: VecDeque<NodeId>
}

impl History {
    pub fn new(capacity: usize, init_state: NodeId) -> Self {
        let mut tape: VecDeque<NodeId> = VecDeque::with_capacity(capacity);
        tape.push_front(init_state);

        Self {
            tape
        }
    }

    pub fn unwind(&mut self) {
        if self.tape.len() > 1 {
            self.tape.pop_back();
        }
    }

    pub fn enqueue(&mut self, state: NodeId) {
        if self.tape.len() % self.tape.capacity() == 0 {
            self.tape.pop_front();
        }
        
        self.tape.push_back(state);
    }

    pub fn state(&self) -> NodeId {
        self.tape[self.tape.len() - 1]
    }
}
