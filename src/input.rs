#[allow(unused_imports)]
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
#[allow(unused_imports)]
use termion::raw::IntoRawMode;

use crate::command::Command;
use crate::cursor::Cursor;
use crate::editor::Editor;
use crate::mode::Mode;

fn move_cursor(cursor: &mut Cursor, dx: i16, dy: i16) {
    let new_x = cursor.x as i16 + dx;
    let new_y = cursor.y as i16 + dy;
    cursor.x = if new_x < 1 { 1 } else { new_x as usize };
    cursor.y = if new_y < 1 { 1 } else { new_y as usize };
}

#[allow(unused_variables)]
pub fn input(
    db: &mut rzdb::Db,
    table_name: &str,
    cursor: &mut Cursor,
    command: &mut Command,
    mode: &mut Mode,
    editor: &mut Editor,
) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    //let c = stdin.keys().next().unwrap();
    if let Some(c) = stdin.keys().next() {
        match mode.clone() {
            Mode::Normal => match c.unwrap() {
                Key::Char('q') => *command = Command::Quit,
                Key::Char('j') => move_cursor(cursor, 0, 1),
                Key::Char('k') => move_cursor(cursor, 0, -1),
                Key::Char('h') => move_cursor(cursor, -1, 0),
                Key::Char('l') => move_cursor(cursor, 1, 0),
                Key::Char('0') => cursor.x = 1,
                Key::Char('$') => cursor.x = db.get_column_names(table_name).len(),
                Key::Char('g') => cursor.y = 1,
                Key::Char('G') => cursor.y = db.select_from(table_name).len(),
                Key::Char('i') => *mode = Mode::Insert,

                Key::Char(c) => println!("{}", c),
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl(c) => println!("*{}", c),
                Key::Esc => println!("ESC"),
                Key::Left => move_cursor(cursor, -1, 0),
                Key::Right => move_cursor(cursor, 1, 0),
                Key::Up => move_cursor(cursor, 0, -1),
                Key::Down => move_cursor(cursor, 0, 1),
                Key::Backspace => println!("×"),
                _ => {}
            },
            Mode::Insert => match c.unwrap() {
                Key::Esc => {
                    *command = Command::ExitEditor;
                    *mode = Mode::Normal;
                }
                Key::Left => editor.left(),
                Key::Right => editor.right(),
                Key::Char(c) => editor.add(c),
                Key::Backspace => editor.backspace(),
                Key::Delete => editor.delete(),
                _ => {}
            },
        }
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
