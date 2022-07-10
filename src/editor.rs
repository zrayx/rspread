pub struct Editor {
    pub line: String,
    pub cur_x: usize,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            line: "".to_string(),
            cur_x: 0,
        }
    }

    fn cur_x_bytes(&self) -> usize {
        self.line.chars().take(self.cur_x).collect::<String>().len()
    }

    pub fn left(&mut self) {
        if self.cur_x > 0 {
            self.cur_x -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cur_x < self.line.chars().count() {
            self.cur_x += 1;
        }
    }

    pub fn home(&mut self) {
        self.cur_x = 0;
    }

    pub fn end(&mut self) {
        self.cur_x = self.line.chars().count();
    }

    pub fn add(&mut self, ch: char) {
        self.line.insert(self.cur_x_bytes(), ch);
        self.cur_x += 1;
    }

    pub fn clear(&mut self) {
        self.line.clear();
        self.cur_x = 0;
    }

    pub fn insert_at(&mut self, old_text: &str, idx: usize) {
        self.line.clear();
        self.line.push_str(old_text);
        self.cur_x = idx.min(self.line.chars().count());
    }

    pub fn backspace(&mut self) {
        if self.cur_x > 0 {
            self.cur_x -= 1;
            self.line.remove(self.cur_x_bytes());
        }
    }

    pub fn delete(&mut self) {
        self.line.remove(self.cur_x_bytes());
    }

    pub fn delete_left_all(&mut self) {
        while self.cur_x > 0 {
            self.backspace();
        }
    }

    pub fn delete_right_all(&mut self) {
        while self.cur_x < self.line.chars().count() {
            self.delete();
        }
    }

    pub fn delete_word(&mut self) {
        while self.cur_x > 0
            && self
                .line
                .chars()
                .nth(self.cur_x - 1)
                .unwrap()
                .is_whitespace()
        {
            self.backspace();
        }
        while self.cur_x > 0
            && !self
                .line
                .chars()
                .nth(self.cur_x - 1)
                .unwrap()
                .is_whitespace()
        {
            self.backspace();
        }
    }

    pub fn get_line(&self) -> String {
        self.line.clone()
    }
}
