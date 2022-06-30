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

    pub fn add(&mut self, ch: char) {
        self.line.insert(self.cur_x, ch);
        self.cur_x += 1;
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

    pub fn backspace(&mut self) {
        if self.cur_x > 0 {
            self.line.remove(self.cur_x - 1);
            self.cur_x -= 1;
        }
    }

    pub fn delete(&mut self) {
        self.line.remove(self.cur_x);
    }
}
