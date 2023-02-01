use arboard::Clipboard;

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

    pub fn len_utf8(&self) -> usize {
        self.line.chars().count()
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

    pub fn indent_left(&mut self) {
        let indent_size = 2;
        let s: String = self.line.chars().take(indent_size).collect();
        for i in s.chars() {
            if i != ' ' {
                return;
            }
            self.line.remove(0);
            if self.cur_x > 0 {
                self.cur_x -= 1;
            }
        }
    }

    pub fn indent_right(&mut self) {
        let indent_size = 2;
        self.line.insert_str(0, &" ".repeat(indent_size));
        self.cur_x += indent_size;
    }

    pub fn word_left(&mut self) {
        if self.cur_x > 0 {
            let mut i = self.cur_x;
            while i > 0 && self.line.chars().nth(i - 1).unwrap().is_whitespace() {
                i -= 1;
            }
            while i > 0 && !self.line.chars().nth(i - 1).unwrap().is_whitespace() {
                i -= 1;
            }
            self.cur_x = i;
        }
    }

    pub fn word_right(&mut self) {
        if self.cur_x < self.line.chars().count() {
            let mut i = self.cur_x;
            while i < self.line.chars().count()
                && !self.line.chars().nth(i).unwrap().is_whitespace()
            {
                i += 1;
            }
            while i < self.line.chars().count() && self.line.chars().nth(i).unwrap().is_whitespace()
            {
                i += 1;
            }
            self.cur_x = i;
        }
    }

    pub fn home(&mut self) {
        self.cur_x = 0;
    }

    pub fn end(&mut self) {
        self.cur_x = self.line.chars().count();
    }

    pub fn add(&mut self, ch: char) {
        let s = match ch {
            '\t' => "    ".to_string(),
            '\r' => " ".to_string(),
            '\n' => " ".to_string(),
            _ => ch.to_string(),
        };
        self.line.insert_str(self.cur_x_bytes(), &s);
        self.cur_x += s.chars().count();
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

    pub fn insert_clipboard(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        if let Ok(text) = clipboard.get_text() {
            for s in text.chars() {
                self.add(s);
            }
        }
    }
}
