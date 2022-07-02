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

    pub fn left(&mut self) {
        if self.cur_x > 0 {
            self.cur_x -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cur_x < self.line.len() {
            self.cur_x += 1;
        }
    }

    pub fn add(&mut self, ch: char) {
        self.line.insert(self.cur_x, ch);
        self.cur_x += 1;
    }

    pub fn clear(&mut self) {
        self.line.clear();
        self.cur_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cur_x > 0 {
            self.line.remove(self.cur_x - 1);
            self.cur_x -= 1;
        }
    }

    pub fn insert_at(&mut self, old_text: &str, idx: usize) {
        self.line.clear();
        self.line.push_str(old_text);
        self.cur_x = idx;
    }

    pub fn insert_end(&mut self, old_text: &str) {
        self.line.clear();
        self.line.push_str(old_text);
        self.cur_x = old_text.len();
    }

    pub fn delete(&mut self) {
        self.line.remove(self.cur_x);
    }
}
