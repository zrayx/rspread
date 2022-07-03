pub enum Command {
    Quit,
    None,
    ExitEditor,
    ExitEditorRight,
    ExitEditorDown,

    InsertStart,
    InsertEnd,
    ChangeCell,
    DeleteCell,

    InsertToday,

    InsertColumn,
    InsertRowAbove,
    InsertRowBelow,
    DeleteLine,
    DeleteColumn,

    YankCell,
    PasteCell,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
