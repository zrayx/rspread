use rzdb::Db;
use std::io::stdout;
// use termion;
// use termion::event::Key;
// use termion::input::TermRead;
//use termion::color::{Bg, Black, Blue, Cyan, Fg, Green, Magenta, Red, White, Yellow};
#[allow(unused_imports)]
use termion::raw::IntoRawMode;

mod render;
use crate::render::render;

fn main() {
    let table_name = "test1";
    let mut db = Db::create("basic", "./db");
    db.create_table(table_name);
    db.create_column(table_name, "name");
    db.create_column(table_name, "value");
    db.insert(table_name, vec!["hello", "world"]);
    db.insert(table_name, vec!["bon jour", "le monde"]);
    db.insert(table_name, vec!["你好", "世界"]); // no proper display of double width unicode characters

    // let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    render(&mut stdout, &db, table_name);
}
