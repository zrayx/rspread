use rzdb::{time::Date, Data, Db};

use common::*;

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
    let args = std::env::args().collect::<Vec<_>>();
    let (mut path, mut db_name, mut table_name) = match args.len() {
        1 => (
            "~/.local/rzdb".to_string(),
            "rspread".to_string(),
            "todo".to_string(),
        ),
        2 => {
            let path = "~/.local/rzdb".to_string();
            let db_name = "rspread".to_string();
            let table_name = args[1].clone();
            (path, db_name, table_name)
        }
        3 => {
            let path = "~/.local/rzdb".to_string();
            let db_name = args[1].clone();
            let table_name = args[2].clone();
            (path, db_name, table_name)
        }
        4 => {
            let path = args[1].clone();
            let db_name = args[2].clone();
            let table_name = args[3].clone();
            (path, db_name, table_name)
        }
        _ => {
            println!("Usage: rzdb [db_path] [db_name] [table_name]");
            println!("Or   : rzdb [db_name] [table_name]");
            println!("Or   : rzdb [table_name]");
            std::process::exit(1);
        }
    };
    let mut db = if let Ok(db) = Db::load(&db_name, &path) {
        db
    } else {
        Db::create(&db_name, &path).unwrap()
    };
    if !db.exists(&table_name) {
        db.create_table(&table_name).unwrap();
        db.create_column(&table_name, "date").unwrap();
        db.create_column(&table_name, "topic").unwrap();
    };

    // copy & paste
    let clipboard_table_name = "clipboard";
    // process input
    let mut cursor = pos::Pos::new(1, 1);
    let mut command = Command::new();
    let mut mode = Mode::new();
    let mut editor = editor::Editor::new();
    let mut message = String::new();
    loop {
        render::render(&db, &table_name, &cursor, &mode, &editor, &message);

        if mode == Mode::Error {
            mode = Mode::Normal;
        }

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
                common::editor_enter(&db, &table_name, &cursor, &mut editor, 0);
            }
            Command::InsertEnd => {
                mode = Mode::Insert;
                editor_enter(&db, &table_name, &cursor, &mut editor, -1);
            }
            Command::ChangeCell => {
                mode = Mode::Insert;
                editor.insert_at("", 0);
            }
            Command::EditorExit
            | Command::EditorExitUp
            | Command::EditorExitDown
            | Command::EditorExitLeft
            | Command::EditorExitRight => {
                mode = Mode::Normal;
                if let Err(e) =
                    editor_exit(&mut db, &table_name, &mut mode, &mut cursor, &mut editor)
                {
                    set_error_message(&e.to_string(), &mut message, &mut mode);
                }
                if command != Command::EditorExit {
                    if command == Command::EditorExitLeft && cursor.x > 1 {
                        cursor.x -= 1;
                    } else if command == Command::EditorExitRight {
                        cursor.x += 1;
                    } else if command == Command::EditorExitUp && cursor.y > 0 {
                        cursor.y -= 1;
                    } else if command == Command::EditorExitDown {
                        cursor.y += 1;
                    }
                    mode = Mode::Insert;
                    editor_enter(&db, &table_name, &cursor, &mut editor, -1);
                }
            }
            Command::CommandLineEnter => {
                mode = Mode::Command;
            }
            Command::CommandLineExit => {
                let line = editor.get_line();
                let mut args = line.split_whitespace();
                if let Some(command) = args.next() {
                    match command {
                        "e" => load_table(
                            &mut args,
                            &mut table_name,
                            &mut cursor,
                            &mut db,
                            &mut editor,
                        ),
                        "ls" => list_tables(&mut table_name, &mut cursor, &mut db, &mut mode),
                        "drop" => {
                            drop_table(args, &mut db, &mut table_name, &mut cursor, &mut mode)
                        }
                        "cd" => {
                            if let Some(arg1) = args.next() {
                                if let Some(arg2) = args.next() {
                                    path = arg1.to_string();
                                    db_name = arg2.to_string();
                                } else {
                                    db_name = arg1.to_string();
                                }
                                match Db::load(&db_name, &path) {
                                    Ok(new_db) => {
                                        db = new_db;
                                    }
                                    Err(e) => {
                                        set_error_message(
                                            &format!(
                                                "Could not load database as {}/{}: {}",
                                                path, db_name, e
                                            ),
                                            &mut message,
                                            &mut mode,
                                        );
                                    }
                                }
                                list_tables(&mut table_name, &mut cursor, &mut db, &mut mode);
                            }
                        }
                        "pwd" => {
                            println!("{}", path);
                            let new_table_name = ".".to_string();
                            set_table(&new_table_name, &mut table_name, &mut cursor);
                            db.create_or_replace_table(&*table_name).unwrap();
                            db.create_column(&*table_name, "name").unwrap();
                            db.create_column(&*table_name, "value").unwrap();
                            db.insert(&table_name, vec!["database path", &path])
                                .unwrap();
                            db.insert(&table_name, vec!["database name", &db_name])
                                .unwrap();
                            mode = Mode::ListReadOnly;
                        }
                        _ => {
                            set_error_message(
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
                            set_table(&new_table_name, &mut table_name, &mut cursor);
                            mode = Mode::Normal;
                        }
                    }
                }
            }
            Command::InsertToday => {
                if cursor.y > 0 {
                    extend_table(&mut db, &table_name, cursor.x, cursor.y).unwrap();
                    db.set_at(
                        &table_name,
                        cursor.y - 1,
                        cursor.x - 1,
                        Data::Date(Date::today()),
                    )
                    .unwrap();
                }
            }
            Command::InsertColumn => {
                let mut column_count = db.get_column_count(&table_name).unwrap();
                while column_count < cursor.x {
                    db.create_column(
                        &table_name,
                        &generate_column_name(&db, &table_name, column_count),
                    )
                    .unwrap();
                    column_count += 1;
                }
                db.insert_column_at(
                    &table_name,
                    &generate_column_name(&db, &table_name, cursor.x),
                    cursor.x - 1,
                )
                .unwrap();
            }
            Command::InsertRowAbove => {
                if cursor.y > 0 && is_cell(&db, &table_name, 0, cursor.y - 1) {
                    db.insert_row_at(&table_name, cursor.y - 1).unwrap();
                }
            }
            Command::InsertRowBelow => {
                if cursor.y > 0 && is_cell(&db, &table_name, 0, cursor.y) {
                    db.insert_row_at(&table_name, cursor.y).unwrap();
                }
                cursor.y += 1;
            }
            Command::DeleteCell => {
                if cursor.y > 0 {
                    if is_cell(&db, &table_name, cursor.x - 1, cursor.y - 1) {
                        db.set_at(&table_name, cursor.y - 1, cursor.x - 1, Data::Empty)
                            .unwrap();
                    }
                } else {
                    let old_column_name = db.get_column_name_at(&table_name, cursor.x - 1).unwrap();
                    let generic_column_name = generate_column_name(&db, &table_name, cursor.x - 1);
                    db.rename_column(&table_name, &old_column_name, &generic_column_name)
                        .unwrap();
                }
            }
            Command::DeleteLine => {
                if cursor.y > 0 && is_cell(&db, &table_name, 0, cursor.y - 1) {
                    db.delete_row_at(&table_name, cursor.y - 1).unwrap();
                }
                if cursor.y > 1 && cursor.y > db.get_row_count(&table_name).unwrap() {
                    cursor.y -= 1;
                }
            }
            Command::DeleteColumn => {
                if is_cell(&db, &table_name, cursor.x - 1, 0) {
                    db.delete_column(
                        &table_name,
                        &db.get_column_name_at(&table_name, cursor.x - 1).unwrap(),
                    )
                    .unwrap();
                }
            }
            Command::YankCell => {
                let column_name = get_column_name_or_generic(cursor.x, &db, &table_name);
                if cursor.y == 0 {
                    db.create_or_replace_table(clipboard_table_name).unwrap();
                    db.create_column(clipboard_table_name, &column_name)
                        .unwrap();
                    db.insert(clipboard_table_name, vec![&column_name]).unwrap();
                } else {
                    let (start, end) = if cursor.y == 0 {
                        (0, 0)
                    } else {
                        (cursor.y - 1, cursor.y)
                    };
                    db.select_into(
                        clipboard_table_name,
                        &table_name,
                        &[&column_name],
                        start,
                        end,
                    )
                    .unwrap();
                }
            }
            Command::PasteCell => {
                if let Ok(cell_data) = db.select_at(clipboard_table_name, 0, 0) {
                    if cursor.y > 0 {
                        extend_table(&mut db, &table_name, cursor.x, cursor.y).unwrap();
                        db.set_at(&table_name, cursor.y - 1, cursor.x - 1, cell_data)
                            .unwrap();
                    } else {
                        let new_name = cell_data.to_string();
                        let old_name = db.get_column_name_at(&table_name, cursor.x - 1).unwrap();
                        db.rename_column(&table_name, &old_name, &new_name).unwrap();
                    }
                }
            }
        }

        if let Err(e) = db.save() {
            set_error_message(
                &format!("Error saving database at {}/{}: {}", path, db_name, e),
                &mut message,
                &mut mode,
            );
        }
    }

    render::cleanup();
}
