pub enum Command {
    Quit,
    None,
    InsertToday,
    InsertColumn,
    InsertRowAbove,
    InsertRowBelow,
    DeleteLine,
    DeleteColumn,
    ExitEditor,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
