use rzdb::{Data, Db};

mod command;
mod cursor;
mod editor;
mod input;
mod mode;
mod render;

use crate::input::input;

fn main() {
    let table_name = "test1";
    let mut db = if let Ok(db) = Db::load("rspread", "./db") {
        db
    } else {
        Db::create("rspread", "./db").unwrap()
    };
    if !db.exists(table_name) {
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
        db.insert(table_name, vec!["2022.07.01", "rzdb: load and save"]);
    };

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
                exit_editor(&mut db, table_name, &mut mode, &mut cursor, &mut editor).unwrap();
            }
        }
        db.save().unwrap();
    }
    render::cleanup();
}

fn extend_table(
    db: &mut Db,
    table_name: &str,
    new_row_count: usize,
    new_column_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_row_count = db.row_count(table_name);
    let old_column_count = db.column_count(table_name);
    for idx in old_column_count..new_column_count {
        db.create_column(table_name, &format!("Column {}", idx + 1));
    }
    for _ in old_row_count..new_row_count {
        let column_count = new_column_count.max(old_column_count);
        db.insert(table_name, vec![""; column_count]);
    }
    Ok(())
}

fn exit_editor(
    db: &mut Db,
    table_name: &str,
    mode: &mut mode::Mode,
    cursor: &mut cursor::Cursor,
    editor: &mut editor::Editor,
) -> Result<(), Box<dyn std::error::Error>> {
    extend_table(db, table_name, cursor.y, cursor.x)?;
    db.set_at(
        table_name,
        cursor.y - 1,
        cursor.x - 1,
        Data::parse(&editor.line),
    )?;
    editor.clear();
    *mode = mode::Mode::Normal;
    Ok(())
}
