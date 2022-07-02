#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Delete,
}

impl Mode {
    pub fn new() -> Mode {
        Mode::Normal
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::Normal => "Normal".to_string(),
                Mode::Insert => "Insert".to_string(),
                Mode::Delete => "Delete".to_string(),
            }
        )
    }
}
