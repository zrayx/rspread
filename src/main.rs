use rzdb::Db;

mod command;
mod cursor;
mod editor;
mod input;
mod mode;
mod render;

use crate::input::input;
fn main() {
    let table_name = "test1";
    let mut db = Db::create("basic", "./db");
    db.create_table(table_name);
    db.create_column(table_name, "date");
    db.create_column(table_name, "topic");
    db.insert(table_name, vec!["2022.06.30", "navigate with cursor"]);
    db.insert(table_name, vec!["2022.06.30", "edit"]);
    db.insert(
        table_name,
        vec![
            "2022.06.30",
            "render the text white, the cursor black on white (editor is black on yellow)",
        ],
    );
    db.insert(
        table_name,
        vec![
            "2022.06.30",
            "render the cursor, row numbers and column names when the cursor is on empty cells",
        ],
    );
    db.insert(table_name, vec!["2022.06.30", "line editor"]);
    db.insert(
        table_name,
        vec!["2022.06.30", "terminal doesn't use all rows and columns"],
    );

    // process input
    let mut cursor = cursor::Cursor::new(1, 1);
    let mut command = command::Command::new();
    let mut mode = mode::Mode::new();
    let mut editor = editor::Editor::new();
    loop {
        render::render(&db, table_name, &cursor, &mode, &editor);
        input(
            &mut db,
            table_name,
            &mut cursor,
            &mut command,
            &mut mode,
            &mut editor,
        );
        match command {
            command::Command::Quit => break,
            command::Command::None => {}
            command::Command::ExitEditor => {
                mode = mode::Mode::Normal;
            }
        }
    }
    render::cleanup();
}
