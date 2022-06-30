use rzdb::Db;
// use termion;
// use termion::event::Key;
// use termion::input::TermRead;
//use termion::color::{Bg, Black, Blue, Cyan, Fg, Green, Magenta, Red, White, Yellow};
#[allow(unused_imports)]
use termion::raw::IntoRawMode;

mod command;
mod cursor;
mod input;
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

    // process input
    let mut cursor = cursor::Cursor::new(1, 1);
    let mut command = command::Command::new();
    loop {
        render::render(&db, table_name, &cursor);
        input(&mut db, &mut cursor, &mut command);
        match command {
            command::Command::Quit => break,
            command::Command::None => {}
        }
    }
    render::cleanup();
}
