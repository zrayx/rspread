#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
}

impl Mode {
    pub fn new() -> Mode {
        Mode::Normal
    }
}
