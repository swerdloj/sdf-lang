pub mod analyze;



pub struct Cursor {
    string: String,
    current: usize,
}

impl Cursor {
    pub fn from_string(input: String) -> Self {
        Cursor {
            string: input,
            current: 0,
        }
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn move_to(&mut self, to: usize) {
        self.current = to;
    }

    pub fn is_finished(&self) -> bool {
        self.string.len() == self.current + 1
    }

    pub fn print(&self) -> &str {
        &self.string[self.current..]
    }

    pub fn current_character(&self) -> &str {
        &self.string[self.current..self.current+1]
    }

    pub fn peek_ahead(&self, amount: usize) -> &str {
        &self.string[self.current+amount..self.current+amount+1]
    }
}