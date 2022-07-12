#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Yank,
    Paste,
    Command,
    Delete,
    ListTables,
    ListDatabases,
    ListReadOnly,
    Error,
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
                Mode::Yank => "Yank".to_string(),
                Mode::Paste => "Paste".to_string(),
                Mode::Delete => "Delete".to_string(),
                Mode::Command => "Command".to_string(),
                Mode::ListTables => "List Tables".to_string(),
                Mode::ListDatabases => "List Databases".to_string(),
                Mode::ListReadOnly => "List Temp Table".to_string(),
                Mode::Error => "Error".to_string(),
            }
        )
    }
}
