pub mod analyze;
pub mod tokenize;


pub enum LexError {

}

struct Cursor {
    string: String,
    current: usize,
}

impl Cursor {
    /// Create a `Cursor` object from a `String`
    pub fn from_string(input: String) -> Self {
        Cursor {
            string: input,
            current: 0,
        }
    }

    /// Move the cursor forward by one
    pub fn advance(&mut self) {
        self.current += 1;
    }

    /// Move the cursor forward by a given amount
    pub fn advance_by(&mut self, by: usize) {
        self.current += by;
    }

    /// Search for a specific character
    /// - Returns `Some(distance to character)` if found
    /// - Returns `None` if not found
    pub fn seek_char(&self, character: char) -> Option<usize> {
        let mut current = self.current_character();
        let mut offset = 0;

        while current != character {
            offset += 1;
            
            if self.current + offset + 1 > self.string.len() {
                return None;
            }

            current = self.peek(offset);
        }

        Some(offset)
    }

    /// Returns the character `ahead` positions ahead of the cursor's current position
    pub fn peek(&self, ahead: usize) -> char {
        self.string.chars().nth(self.current + ahead).unwrap()
    }

    /// Moves the cursor to the given index
    pub fn move_to(&mut self, to: usize) {
        self.current = to;
    }

    /// Moves cursor to end of string
    pub fn move_to_end(&mut self) {
        self.current = self.string.len();
    }

    /// Whether the cursor has passed the string
    pub fn is_finished(&self) -> bool {
        self.string.len() == self.current
    }

    /// Returns the substring from the cursor position to the end of the string
    pub fn print(&self) -> &str {
        &self.string[self.current..]
    }

    pub fn print_ahead(&self, ahead: usize) -> &str {
        &self.string[self.current..self.current+ahead]
    }

    /// Returns the substring from the given position to the end of the string
    pub fn print_from(&self, from: usize) -> &str {
        &self.string[from..]
    }

    /// Returns the character at the cursor's position
    pub fn current_character(&self) -> char {
        self.string.chars().nth(self.current).unwrap()
    }
}