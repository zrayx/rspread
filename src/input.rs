use crate::command::Command;
use crate::cursor::Cursor;
#[allow(unused_imports)]
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
#[allow(unused_imports)]
use termion::raw::IntoRawMode;

fn move_cursor(cursor: &mut Cursor, dx: i16, dy: i16) {
    let new_x = cursor.x as i16 + dx;
    let new_y = cursor.y as i16 + dy;
    cursor.x = if new_x < 1 { 1 } else { new_x as u16 };
    cursor.y = if new_y < 1 { 1 } else { new_y as u16 };
}

#[allow(unused_variables)]
pub fn input(db: &mut rzdb::Db, cursor: &mut Cursor, command: &mut Command) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    //let c = stdin.keys().next().unwrap();
    if let Some(c) = stdin.keys().next() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine
        )
        .unwrap();

        match c.unwrap() {
            Key::Char('q') => *command = Command::Quit,
            Key::Char('j') => move_cursor(cursor, 0, 1),
            Key::Char('k') => move_cursor(cursor, 0, -1),
            Key::Char('h') => move_cursor(cursor, -1, 0),
            Key::Char('l') => move_cursor(cursor, 1, 0),

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
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
