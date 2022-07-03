use rzdb::{time::Date, Data, Db};

mod command;
mod common;
mod cursor;
mod editor;
mod input;
mod mode;
mod render;

use crate::command::Command;
use crate::input::input;

fn main() {
    let table_name = "todo";
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
    };

    // copy & paste
    let mut copy_buffer = Data::Empty;
    // process input
    let mut cursor = cursor::Cursor::new(1, 1);
    let mut command = Command::new();
    let mut mode = mode::Mode::new();
    let mut editor = editor::Editor::new();
    loop {
        render::render(&db, table_name, &cursor, &mode, &editor);
        input(
            &db,
            table_name,
            &mut cursor,
            &mut command,
            &mut mode,
            &mut editor,
        );
        match command {
            Command::Quit => break,
            Command::None => {}
            Command::InsertStart => {
                mode = mode::Mode::Insert;
                enter_editor(&db, table_name, &cursor, &mut editor, 0);
            }
            Command::InsertEnd => {
                mode = mode::Mode::Insert;
                enter_editor(&db, table_name, &cursor, &mut editor, -1);
            }
            Command::ChangeCell => {
                mode = mode::Mode::Insert;
                editor.insert_at("", 0);
            }
            Command::ExitEditor => {
                mode = mode::Mode::Normal;
                exit_editor(&mut db, table_name, &mut mode, &mut cursor, &mut editor).unwrap();
            }
            Command::ExitEditorRight => {
                mode = mode::Mode::Normal;
                exit_editor(&mut db, table_name, &mut mode, &mut cursor, &mut editor).unwrap();
                cursor.x += 1;
            }
            Command::ExitEditorDown => {
                mode = mode::Mode::Normal;
                exit_editor(&mut db, table_name, &mut mode, &mut cursor, &mut editor).unwrap();
                cursor.y += 1;
            }

            Command::InsertToday => {
                if cursor.y > 0 {
                    if db.get_row_count(table_name) < cursor.y {
                        let column_count = db.get_column_count(table_name);
                        db.insert(table_name, vec![""; column_count]);
                    }
                    db.set_at(
                        table_name,
                        cursor.y - 1,
                        cursor.x - 1,
                        Data::parse(&Date::today().to_string()),
                    )
                    .unwrap();
                }
            }
            Command::InsertColumn => {
                let mut column_count = db.get_column_count(table_name);
                while column_count < cursor.x {
                    db.create_column(
                        table_name,
                        &common::generate_column_name(&db, table_name, column_count),
                    );
                    column_count += 1;
                }
                db.insert_column_at(
                    table_name,
                    &common::generate_column_name(&db, table_name, cursor.x),
                    cursor.x - 1,
                );
            }
            Command::InsertRowAbove => {
                if cursor.y > 0 && common::is_cell(&db, table_name, 0, cursor.y - 1) {
                    db.insert_row_at(table_name, cursor.y - 1);
                }
            }
            Command::InsertRowBelow => {
                if cursor.y > 0 && common::is_cell(&db, table_name, 0, cursor.y) {
                    db.insert_row_at(table_name, cursor.y);
                }
                cursor.y += 1;
            }
            Command::DeleteCell => {
                if cursor.y > 0 {
                    if common::is_cell(&db, table_name, cursor.x - 1, cursor.y - 1) {
                        db.set_at(table_name, cursor.y - 1, cursor.x - 1, Data::Empty)
                            .unwrap();
                    }
                } else {
                    let old_column_name = db.get_column_name_at(table_name, cursor.x - 1);
                    let generic_column_name =
                        common::generate_column_name(&db, table_name, cursor.x - 1);
                    db.rename_column(table_name, &old_column_name, &generic_column_name);
                }
            }
            Command::DeleteLine => {
                if cursor.y > 0 && common::is_cell(&db, table_name, 0, cursor.y - 1) {
                    db.delete_row_at(table_name, cursor.y - 1);
                }
            }
            Command::DeleteColumn => {
                if common::is_cell(&db, table_name, cursor.x - 1, 0) {
                    db.delete_column(table_name, &db.get_column_names(table_name)[cursor.x - 1]);
                }
            }
            Command::YankCell => {
                copy_buffer = if cursor.y == 0 {
                    Data::parse(&common::get_column_name_or_generic(
                        cursor.x, &db, table_name,
                    ))
                } else if common::is_cell(&db, table_name, cursor.x - 1, cursor.y - 1) {
                    db.select_at(table_name, cursor.x - 1, cursor.y - 1)
                } else {
                    Data::Empty
                };
            }
            Command::PasteCell => {
                if cursor.y > 0 {
                    extend_table(&mut db, table_name, cursor.x, cursor.y).unwrap();
                    db.set_at(table_name, cursor.y - 1, cursor.x - 1, copy_buffer.clone())
                        .unwrap();
                }
            }
        }
        db.save().unwrap();
    }

    render::cleanup();
}

fn extend_table(
    db: &mut Db,
    table_name: &str,
    new_column_count: usize,
    new_row_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_row_count = db.get_row_count(table_name);
    let old_column_count = db.get_column_count(table_name);
    for idx in old_column_count..new_column_count {
        db.create_column(
            table_name,
            &common::generate_column_name(db, table_name, idx + 1),
        );
    }
    for _ in old_row_count..new_row_count {
        let column_count = new_column_count.max(old_column_count);
        db.insert(table_name, vec![""; column_count]);
    }
    Ok(())
}

fn enter_editor(
    db: &Db,
    table_name: &str,
    cursor: &cursor::Cursor,
    editor: &mut editor::Editor,
    cursor_x: i32,
) {
    let column_count = db.get_column_count(table_name);
    let old_text = if cursor.y == 0 {
        if cursor.x > column_count {
            common::generate_column_name(db, table_name, cursor.x)
        } else {
            db.get_column_names(table_name)[cursor.x - 1].clone()
        }
    } else if common::is_cell(db, table_name, cursor.x - 1, cursor.y - 1) {
        db.select_at(table_name, cursor.x - 1, cursor.y - 1)
            .to_string()
    } else {
        "".to_string()
    };
    if cursor_x < 0 {
        let len = old_text.len() as i32;
        if len - cursor_x >= 0 {
            editor.insert_at(&old_text, (len - cursor_x + 1) as usize);
        } else {
            editor.insert_at(&old_text, 0);
        }
    } else {
        editor.insert_at(&old_text, cursor_x as usize);
    }
}

fn exit_editor(
    db: &mut Db,
    table_name: &str,
    mode: &mut mode::Mode,
    cursor: &mut cursor::Cursor,
    editor: &mut editor::Editor,
) -> Result<(), Box<dyn std::error::Error>> {
    extend_table(db, table_name, cursor.x, cursor.y)?;
    if cursor.y == 0 {
        // column name
        let old_column_name = common::get_column_name_or_generic(cursor.x, db, table_name);
        let new_column_name = editor.line();
        db.rename_column(table_name, &old_column_name, &new_column_name);
    } else {
        db.set_at(
            table_name,
            cursor.y - 1,
            cursor.x - 1,
            Data::parse(&editor.line),
        )?;
    }
    editor.clear();
    *mode = mode::Mode::Normal;
    Ok(())
}
