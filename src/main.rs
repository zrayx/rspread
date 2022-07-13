use inotify::{Inotify, WatchMask};

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
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    let args = std::env::args().collect::<Vec<_>>();
    let (mut db_dir, mut db_name, mut table_name) = match args.len() {
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
    // TODO: check if database exists; if it exists, return error on failure to load
    let mut db = if let Ok(db) = Db::load(&db_name, &db_dir) {
        db
    } else {
        Db::create(&db_name, &db_dir).unwrap()
    };
    if !db.exists(&table_name) {
        db.create_table(&table_name).unwrap();
        db.create_column(&table_name, "date").unwrap();
        db.create_column(&table_name, "topic").unwrap();
    };

    // setup inotify for db_dir (reload database on change)
    let mut watch_descriptor = inotify
        .add_watch(
            &format!("{}/{}", db.get_db_path(), db_name),
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
        )
        .unwrap();

    // macro to remove the watch descriptor and renew it
    macro_rules! renew_watch_descriptor {
        () => {
            inotify.rm_watch(watch_descriptor).unwrap();
            watch_descriptor = inotify
                .add_watch(
                    &format!("{}/{}", db.get_db_path(), db_name),
                    WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
                )
                .unwrap();
        };
    }

    // copy & paste
    let clipboard_table_name = "clipboard";
    // process input
    let mut cursor = pos::Pos::new(1, 1);
    let mut command = Command::new();
    let mut mode = Mode::new();
    let mut editor = editor::Editor::new();
    let mut message = String::new();
    loop {
        // render screen
        render::render(&db, &table_name, &cursor, &mode, &editor, &message);

        // reset error message display
        if mode == Mode::Error {
            mode = Mode::Normal;
        }

        // get user input
        input(
            &db,
            &table_name,
            &mut cursor,
            &mut command,
            &mut mode,
            &mut editor,
            &mut message,
        );

        // reload database if files in it have changed
        let mut buffer = [0u8; 1024];
        if let Ok(events) = inotify.read_events(&mut buffer) {
            if events.last().is_some() {
                db = Db::load(&db_name, &db_dir).unwrap();
                set_error_message("Files changed, reloaded database", &mut message, &mut mode);
            }
        }

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
                        "lsdb" => {
                            list_databases(&mut table_name, &mut cursor, &mut db, &mut mode);
                            renew_watch_descriptor!();
                        }

                        "drop" => {
                            drop_table(args, &mut db, &mut table_name, &mut cursor, &mut mode);
                            consume_inotify_events(&mut inotify, buffer);
                        }
                        "cd" => {
                            if let Some(arg1) = args.next() {
                                if let Some(arg2) = args.next() {
                                    db_dir = arg1.to_string();
                                    db_name = arg2.to_string();
                                } else {
                                    db_name = arg1.to_string();
                                }
                                match Db::load(&db_name, &db_dir) {
                                    Ok(new_db) => {
                                        db = new_db;
                                    }
                                    Err(e) => {
                                        set_error_message(
                                            &format!(
                                                "Could not load database as {}/{}: {}",
                                                db_dir, db_name, e
                                            ),
                                            &mut message,
                                            &mut mode,
                                        );
                                    }
                                }
                                list_tables(&mut table_name, &mut cursor, &mut db, &mut mode);
                            }
                            renew_watch_descriptor!();
                        }
                        "pwd" => {
                            let new_table_name = ".".to_string();
                            set_table(&new_table_name, &mut table_name, &mut cursor);
                            db.create_or_replace_table(&*table_name).unwrap();
                            db.create_column(&*table_name, "name").unwrap();
                            db.create_column(&*table_name, "value").unwrap();
                            db.insert(&table_name, vec!["database path", &db_dir])
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
            Command::ListTablesEnter | Command::ListDatabasesEnter => {
                if cursor.y > 0 {
                    if let Ok(selected_name) = db.select_at(&table_name, 0, cursor.y - 1) {
                        let new_name = selected_name.to_string();
                        if new_name != "." {
                            match command {
                                Command::ListDatabasesEnter => {
                                    db_name = new_name;
                                    match Db::load(&db_name, &db_dir) {
                                        Ok(new_db) => {
                                            db = new_db;
                                        }
                                        Err(e) => {
                                            set_error_message(
                                                &format!(
                                                    "Could not load database as {}/{}: {}",
                                                    db_dir, db_name, e
                                                ),
                                                &mut message,
                                                &mut mode,
                                            );
                                        }
                                    }
                                    list_tables(&mut table_name, &mut cursor, &mut db, &mut mode);
                                    mode = Mode::ListTables;
                                }
                                Command::ListTablesEnter => {
                                    set_table(&new_name, &mut table_name, &mut cursor);
                                    mode = Mode::Normal;
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
            Command::PasteToday => {
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
            Command::InsertEmptyColumn => {
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
            Command::InsertEmptyRowAbove => {
                if cursor.y > 0 && is_cell(&db, &table_name, 0, cursor.y - 1) {
                    db.insert_empty_row_at(&table_name, cursor.y - 1).unwrap();
                }
            }
            Command::InsertEmptyRowBelow => {
                if cursor.y > 0 && is_cell(&db, &table_name, 0, cursor.y) {
                    db.insert_empty_row_at(&table_name, cursor.y).unwrap();
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
            Command::YankCell | Command::YankRow | Command::YankColumn => {
                let (start_x, end_x, start_y, end_y) = match command {
                    Command::YankCell => (cursor.x, cursor.x + 1, cursor.y, cursor.y + 1),
                    Command::YankRow => (
                        1,
                        db.get_column_count(&table_name).unwrap() + 1,
                        cursor.y,
                        cursor.y + 1,
                    ),
                    Command::YankColumn => (
                        cursor.x,
                        cursor.x + 1,
                        1,
                        db.get_row_count(&table_name).unwrap() + 1,
                    ),
                    _ => unreachable!(),
                };
                yank(
                    start_x,
                    end_x,
                    start_y,
                    end_y,
                    &mut db,
                    &table_name,
                    clipboard_table_name,
                )
            }
            Command::PasteReplace | Command::PasteBefore | Command::PasteAfter => paste(
                &mut db,
                &table_name,
                clipboard_table_name,
                &mut cursor,
                &command,
            ),
        }

        if let Err(e) = db.save() {
            set_error_message(
                &format!("Error saving database at {}/{}: {}", db_dir, db_name, e),
                &mut message,
                &mut mode,
            );
        }
        consume_inotify_events(&mut inotify, buffer);
    }

    render::cleanup();
}

fn consume_inotify_events(inotify: &mut Inotify, mut buffer: [u8; 1024]) {
    // consume events generated by save()
    let _ = inotify.read_events(&mut buffer);
}
