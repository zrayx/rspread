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
