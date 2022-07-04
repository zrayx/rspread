use rzdb::{time::Date, Data, Db};

mod command;
mod common;
mod editor;
mod input;
mod mode;
mod pos;
mod render;

use command::Command;
use input::input;
use mode::Mode;

fn main() {
    let mut table_name = "todo".to_string();
    let mut db = if let Ok(db) = Db::load("rspread", "./db") {
        db
    } else {
        Db::create("rspread", "./db").unwrap()
    };
    if !db.exists(&table_name) {
        db.create_table(&table_name).unwrap();
        db.create_column(&table_name, "date").unwrap();
        db.create_column(&table_name, "topic").unwrap();
        db.insert(&table_name, vec!["2022.06.30", "navigate with cursor"])
            .unwrap();
    };

    // copy & paste
    let mut copy_buffer = Data::Empty;
    // process input
    let mut cursor = pos::Pos::new(1, 1);
    let mut command = Command::new();
    let mut mode = Mode::new();
    let mut editor = editor::Editor::new();
    let mut message = String::new();
    loop {
        render::render(&db, &table_name, &cursor, &mode, &editor, &message);
        input(
            &db,
            &table_name,
            &mut cursor,
            &mut command,
            &mut mode,
            &mut editor,
            &mut message,
        );
        match command {
            Command::Quit => break,
            Command::None => {}
            Command::InsertStart => {
                mode = Mode::Insert;
                editor_enter(&db, &table_name, &cursor, &mut editor, 0);
            }
            Command::InsertEnd => {
                mode = Mode::Insert;
                editor_enter(&db, &table_name, &cursor, &mut editor, -1);
            }
            Command::ChangeCell => {
                mode = Mode::Insert;
                editor.insert_at("", 0);
            }
            Command::EditorExit => {
                mode = Mode::Normal;
                editor_exit(&mut db, &table_name, &mut mode, &mut cursor, &mut editor).unwrap();
            }
            Command::EditorExitRight => {
                mode = Mode::Normal;
                editor_exit(&mut db, &table_name, &mut mode, &mut cursor, &mut editor).unwrap();
                cursor.x += 1;
            }
            Command::EditorExitDown => {
                mode = Mode::Normal;
                editor_exit(&mut db, &table_name, &mut mode, &mut cursor, &mut editor).unwrap();
                cursor.y += 1;
            }
            Command::CommandLineEnter => {
                mode = Mode::Command;
            }
            Command::CommandLineExit => {
                let line = editor.get_line();
                let mut args = line.split_whitespace();
                if let Some(command) = args.next() {
                    match command {
                        "e" => {
                            if let Some(arg1) = args.next() {
                                // set table_name to new arg
                                let new_table_name = arg1.to_string();
                                common::set_table(&new_table_name, &mut table_name, &mut cursor);
                                if !db.exists(&table_name) {
                                    db.create_table(&table_name).unwrap();
                                }
                                editor.clear();
                                cursor = pos::Pos::new(1, 1);
                            }
                        }
                        "ls" => list_tables(&mut table_name, &mut cursor, &mut db, &mut mode),
                        "drop" => {
                            if let Some(arg1) = args.next() {
                                let name = arg1.to_string();
                                db.drop_table(&name).unwrap();
                                list_tables(&mut table_name, &mut cursor, &mut db, &mut mode);
                            }
                        }
                        _ => {
                            common::set_error_message(
                                &format!("Unknown command: {}", command),
                                &mut message,
                                &mut mode,
                            );
                        }
                    }
                }
                if mode == Mode::Command {
                    mode = Mode::Normal;
                }
                editor.clear();
            }
            Command::ListTablesEnter => {
                if cursor.y > 0 {
                    if let Ok(selected_table_name) = db.select_at(&table_name, 0, cursor.y - 1) {
                        let new_table_name = selected_table_name.to_string();
                        if new_table_name != "." {
                            common::set_table(&new_table_name, &mut table_name, &mut cursor);
                            mode = Mode::Normal;
                        }
                    }
                }
            }
            Command::InsertToday => {
                if cursor.y > 0 {
                    if db.get_row_count(&table_name).unwrap() < cursor.y {
                        let column_count = db.get_column_count(&table_name).unwrap();
                        db.insert(&table_name, vec![""; column_count]).unwrap();
                    }
                    db.set_at(
                        &table_name,
                        cursor.y - 1,
                        cursor.x - 1,
                        Data::parse(&Date::today().to_string()),
                    )
                    .unwrap();
                }
            }
            Command::InsertColumn => {
                let mut column_count = db.get_column_count(&table_name).unwrap();
                while column_count < cursor.x {
                    db.create_column(
                        &table_name,
                        &common::generate_column_name(&db, &table_name, column_count),
                    )
                    .unwrap();
                    column_count += 1;
                }
                db.insert_column_at(
                    &table_name,
                    &common::generate_column_name(&db, &table_name, cursor.x),
                    cursor.x - 1,
                )
                .unwrap();
            }
            Command::InsertRowAbove => {
                if cursor.y > 0 && common::is_cell(&db, &table_name, 0, cursor.y - 1) {
                    db.insert_row_at(&table_name, cursor.y - 1).unwrap();
                }
            }
            Command::InsertRowBelow => {
                if cursor.y > 0 && common::is_cell(&db, &table_name, 0, cursor.y) {
                    db.insert_row_at(&table_name, cursor.y).unwrap();
                }
                cursor.y += 1;
            }
            Command::DeleteCell => {
                if cursor.y > 0 {
                    if common::is_cell(&db, &table_name, cursor.x - 1, cursor.y - 1) {
                        db.set_at(&table_name, cursor.y - 1, cursor.x - 1, Data::Empty)
                            .unwrap();
                    }
                } else {
                    let old_column_name = db.get_column_name_at(&table_name, cursor.x - 1).unwrap();
                    let generic_column_name =
                        common::generate_column_name(&db, &table_name, cursor.x - 1);
                    db.rename_column(&table_name, &old_column_name, &generic_column_name)
                        .unwrap();
                }
            }
            Command::DeleteLine => {
                if cursor.y > 0 && common::is_cell(&db, &table_name, 0, cursor.y - 1) {
                    db.delete_row_at(&table_name, cursor.y - 1).unwrap();
                }
            }
            Command::DeleteColumn => {
                if common::is_cell(&db, &table_name, cursor.x - 1, 0) {
                    db.delete_column(
                        &table_name,
                        &db.get_column_name_at(&table_name, cursor.x - 1).unwrap(),
                    )
                    .unwrap();
                }
            }
            Command::YankCell => {
                copy_buffer = if cursor.y == 0 {
                    Data::parse(&common::get_column_name_or_generic(
                        cursor.x,
                        &db,
                        &table_name,
                    ))
                } else if common::is_cell(&db, &table_name, cursor.x - 1, cursor.y - 1) {
                    db.select_at(&table_name, cursor.x - 1, cursor.y - 1)
                        .unwrap()
                } else {
                    Data::Empty
                };
            }
            Command::PasteCell => {
                if cursor.y > 0 {
                    extend_table(&mut db, &table_name, cursor.x, cursor.y).unwrap();
                    db.set_at(&table_name, cursor.y - 1, cursor.x - 1, copy_buffer.clone())
                        .unwrap();
                }
            }
        }
        if let Err(e) = db.save() {
            common::set_error_message(&format!("{}", e), &mut message, &mut mode);
        }
    }

    render::cleanup();
}

fn list_tables(table_name: &mut String, cursor: &mut pos::Pos, db: &mut Db, mode: &mut Mode) {
    let new_table_name = ".".to_string();
    common::set_table(&new_table_name, table_name, cursor);
    db.create_or_replace_table(&*table_name).unwrap();
    db.create_column(&*table_name, "name").unwrap();
    let mut table_names = db.get_table_names();
    table_names.sort();
    for table in table_names {
        if &table != "." {
            db.insert(&*table_name, vec![&table]).unwrap();
        }
    }
    *mode = Mode::ListTables;
}

fn extend_table(
    db: &mut Db,
    table_name: &str,
    new_column_count: usize,
    new_row_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_row_count = db.get_row_count(table_name).unwrap();
    let old_column_count = db.get_column_count(table_name).unwrap();
    for idx in old_column_count..new_column_count {
        db.create_column(
            table_name,
            &common::generate_column_name(db, table_name, idx + 1),
        )
        .unwrap();
    }
    for _ in old_row_count..new_row_count {
        let column_count = new_column_count.max(old_column_count);
        db.insert(table_name, vec![""; column_count]).unwrap();
    }
    Ok(())
}

fn editor_enter(
    db: &Db,
    table_name: &str,
    cursor: &pos::Pos,
    editor: &mut editor::Editor,
    cursor_x: i32,
) {
    let column_count = db.get_column_count(table_name).unwrap();
    let old_text = if cursor.y == 0 {
        if cursor.x > column_count {
            common::generate_column_name(db, table_name, cursor.x)
        } else {
            db.get_column_name_at(table_name, cursor.x - 1).unwrap()
        }
    } else if common::is_cell(db, table_name, cursor.x - 1, cursor.y - 1) {
        db.select_at(table_name, cursor.x - 1, cursor.y - 1)
            .unwrap()
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

fn editor_exit(
    db: &mut Db,
    table_name: &str,
    mode: &mut Mode,
    cursor: &mut pos::Pos,
    editor: &mut editor::Editor,
) -> Result<(), Box<dyn std::error::Error>> {
    extend_table(db, table_name, cursor.x, cursor.y)?;
    if cursor.y == 0 {
        // column name
        let old_column_name = common::get_column_name_or_generic(cursor.x, db, table_name);
        let new_column_name = editor.get_line();
        db.rename_column(table_name, &old_column_name, &new_column_name)
            .unwrap();
    } else {
        db.set_at(
            table_name,
            cursor.y - 1,
            cursor.x - 1,
            Data::parse(&editor.line),
        )?;
    }
    editor.clear();
    *mode = Mode::Normal;
    Ok(())
}
