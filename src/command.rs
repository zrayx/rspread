#[derive(Clone, PartialEq)]
pub enum Command {
    Quit,
    None,

    EditorExit,
    EditorExitLeft,
    EditorExitRight,
    EditorExitUp,
    EditorExitDown,

    CommandLineEnter,
    CommandLineExit,

    ListTablesEnter,

    InsertStart,
    InsertEnd,
    ChangeCell,
    DeleteCell,

    PasteToday,

    InsertEmptyColumn,
    InsertEmptyRowAbove,
    InsertEmptyRowBelow,
    DeleteLine,
    DeleteColumn,

    YankCell,
    YankRow,
    YankColumn,
    PasteCell,
    PasteRow,
    PasteColumn,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
