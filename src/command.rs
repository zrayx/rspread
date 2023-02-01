#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Command {
    Quit,
    None,
    PreviousFile,

    EditorExit,
    EditorExitLeft,
    EditorExitRight,
    EditorExitUp,
    EditorExitDown,
    EditorNewLine,

    CommandLineEnter,
    CommandLineExit,

    ListTablesEnter,
    ListDatabasesEnter,

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
    IndentLeft,
    IndentRight,

    YankCell,
    YankRow,
    YankColumn,
    PasteReplace,
    PasteBefore,
    PasteAfter,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
