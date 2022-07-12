#[allow(unused_imports)]
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
#[allow(unused_imports)]
use termion::raw::IntoRawMode;

use crate::command::Command;
use crate::common;
use crate::editor::Editor;
use crate::mode::Mode;
use crate::pos::Pos;

fn move_cursor(cursor: &mut Pos, dx: i16, dy: i16) {
    let new_x = cursor.x as i16 + dx;
    let new_y = cursor.y as i16 + dy;
    cursor.x = if new_x < 1 { 1 } else { new_x as usize };
    cursor.y = if new_y < 0 { 0 } else { new_y as usize };
}

#[allow(unused_variables)]
pub fn input(
    db: &rzdb::Db,
    table_name: &str,
    cursor: &mut Pos,
    command: &mut Command,
    mode: &mut Mode,
    editor: &mut Editor,
    message: &mut String,
) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    *command = Command::None;
    //let c = stdin.keys().next().unwrap();
    if let Some(c) = stdin.keys().next() {
        let c = c.unwrap();
        match mode.clone() {
            Mode::Normal => match c {
                Key::Char('q') => *command = Command::Quit,

                Key::Char('j') => move_cursor(cursor, 0, 1),
                Key::Char('k') => move_cursor(cursor, 0, -1),
                Key::Char('h') => move_cursor(cursor, -1, 0),
                Key::Char('l') => move_cursor(cursor, 1, 0),
                Key::Left => move_cursor(cursor, -1, 0),
                Key::Right => move_cursor(cursor, 1, 0),
                Key::Up => move_cursor(cursor, 0, -1),
                Key::Down => move_cursor(cursor, 0, 1),
                Key::Char('\t') => move_cursor(cursor, 1, 0),
                Key::BackTab => move_cursor(cursor, -1, 0),
                Key::Char('\n') => move_cursor(cursor, 0, 1),

                Key::Char('0') | Key::Home => cursor.x = 1,
                Key::Char('$') | Key::End => {
                    cursor.x = db.get_column_names(table_name).unwrap().len()
                }
                Key::Char('g') => cursor.y = 1,
                Key::Char('G') => cursor.y = db.select_from(table_name).unwrap().len(),

                Key::Char('.') => *command = Command::PasteToday,
                Key::Char('I') => *command = Command::InsertEmptyColumn,
                Key::Char('O') => *command = Command::InsertEmptyRowAbove,
                Key::Char('o') => *command = Command::InsertEmptyRowBelow,

                Key::Char('i') => *command = Command::InsertStart,
                Key::Char('a') => *command = Command::InsertEnd,
                Key::Char('x') => *command = Command::DeleteCell,
                Key::Delete => *command = Command::DeleteCell,
                Key::Char('d') => *mode = Mode::Delete,
                Key::Char('C') => *command = Command::ChangeCell,

                Key::Char(':') => *command = Command::CommandLineEnter,

                Key::Ctrl('c') => *command = Command::YankCell,
                Key::Ctrl('v') => *command = Command::PasteCell,
                Key::Char('y') => *mode = Mode::Yank,
                Key::Char('p') => *mode = Mode::Paste,
                Key::Char('Y') => *command = Command::YankRow,
                Key::Char('P') => *command = Command::PasteRow,

                // Key::Backspace => println!("×"),
                // Key::Esc => println!("ESC"),
                // Key::Char(c) => println!("{}", c),
                // Key::Alt(c) => println!("^{}", c),
                _ => common::set_error_message(&format!("Unknown key {:?}", c), message, mode),
            },

            Mode::Insert | Mode::Command => match c {
                Key::Esc
                | Key::Char('\t')
                | Key::Char('\n')
                | Key::BackTab
                | Key::Up
                | Key::Down => {
                    match mode {
                        Mode::Insert => {
                            *command = match c {
                                Key::Esc => Command::EditorExit,
                                Key::Char('\t') => Command::EditorExitRight,
                                Key::BackTab => Command::EditorExitLeft,
                                Key::Up => Command::EditorExitUp,
                                Key::Down | Key::Char('\n') => Command::EditorExitDown,
                                _ => Command::None,
                            };
                        }
                        Mode::Command => {
                            *command = if c == Key::Char('\n') {
                                Command::CommandLineExit
                            } else {
                                Command::None
                            };
                        }
                        _ => common::set_error_message(
                            &format!("Mode {:?} should not appear here", mode),
                            message,
                            mode,
                        ),
                    }
                    if *mode != Mode::Error {
                        *mode = Mode::Normal;
                    }
                }
                Key::Ctrl('a') | Key::Home => editor.home(),
                Key::Ctrl('e') | Key::End => editor.end(),
                Key::Ctrl('u') => editor.delete_left_all(),
                Key::Ctrl('k') => editor.delete_right_all(),
                Key::Ctrl('w') => editor.delete_word(),
                Key::Left | Key::Ctrl('b') => editor.left(),
                Key::Right | Key::Ctrl('f') => editor.right(),
                Key::Char(c) => editor.add(c),
                Key::Backspace | Key::Ctrl('h') => editor.backspace(),
                Key::Delete => editor.delete(),
                _ => {}
            },

            Mode::Yank => match c {
                Key::Esc => *mode = Mode::Normal,
                Key::Char('l') | Key::Char('y') => {
                    *mode = Mode::Normal;
                    *command = Command::YankRow;
                }
                Key::Char('c') => {
                    *mode = Mode::Normal;
                    *command = Command::YankColumn;
                }
                _ => {}
            },

            Mode::Paste => match c {
                Key::Esc => *mode = Mode::Normal,
                Key::Char('l') | Key::Char('p') => {
                    *mode = Mode::Normal;
                    *command = Command::PasteRow;
                }
                Key::Char('c') => {
                    *mode = Mode::Normal;
                    *command = Command::PasteColumn;
                }
                _ => {}
            },

            Mode::Delete => {
                match c {
                    Key::Char('d') => *command = Command::DeleteLine,
                    Key::Char('c') => *command = Command::DeleteColumn,
                    _ => {}
                }
                *mode = Mode::Normal;
            }

            Mode::ListReadOnly | Mode::ListTables | Mode::ListDatabases => match c {
                Key::Char('j') => move_cursor(cursor, 0, 1),
                Key::Char('k') => move_cursor(cursor, 0, -1),
                Key::Up => move_cursor(cursor, 0, -1),
                Key::Down => move_cursor(cursor, 0, 1),
                Key::Char('\n') => match *mode {
                    Mode::ListTables => *command = Command::ListTablesEnter,
                    Mode::ListDatabases => *command = Command::ListDatabasesEnter,
                    _ => {}
                },
                Key::Char(':') => *command = Command::CommandLineEnter,
                _ => {}
            },

            Mode::Error => {
                *mode = Mode::Normal;
                *message = "".to_string();
            }
        }
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
