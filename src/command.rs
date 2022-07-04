pub enum Command {
    Quit,
    None,

    EditorExit,
    EditorExitRight,
    EditorExitDown,

    CommandLineEnter,
    CommandLineExit,

    ListTablesEnter,

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
