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

    pub fn home(&mut self) {
        self.cur_x = 0;
    }

    pub fn end(&mut self) {
        self.cur_x = self.line.len();
    }

    pub fn add(&mut self, ch: char) {
        self.line.insert(self.cur_x, ch);
        self.cur_x += 1;
    }

    pub fn clear(&mut self) {
        self.line.clear();
        self.cur_x = 0;
    }

    pub fn insert_at(&mut self, old_text: &str, idx: usize) {
        self.line.clear();
        self.line.push_str(old_text);
        self.cur_x = idx.min(self.line.len());
    }

    pub fn backspace(&mut self) {
        if self.cur_x > 0 {
            self.line.remove(self.cur_x - 1);
            self.cur_x -= 1;
        }
    }

    pub fn delete(&mut self) {
        self.line.remove(self.cur_x);
    }

    pub fn delete_left_all(&mut self) {
        while self.cur_x > 0 {
            self.backspace();
        }
    }

    pub fn delete_right_all(&mut self) {
        while self.cur_x < self.line.len() {
            self.delete();
        }
    }

    pub fn get_line(&self) -> String {
        self.line.clone()
    }
}
