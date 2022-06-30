use rzdb::Db;
use termion::color::{Bg, Black, Fg, Green, Red, Reset, White};
use termion::cursor::Goto;
use termion::raw::RawTerminal;

use std::io::Write;
// use termion;
// use termion::event::Key;
// use termion::input::TermRead;
//use termion::color::{Bg, Black, Blue, Cyan, Fg, Green, Magenta, Red, White, Yellow};

pub fn render(stdout: &mut RawTerminal<std::io::Stdout>, db: &Db, table_name: &str) {
    let pad = |s: &str, width: u16| {
        let mut s = s.to_string();
        while (s.len() as u16) < width {
            s.push(' ');
        }
        s
    };

    let offset_left = 6; // room for row id
    let offset_top = 0; // room for status line+column headers
    let terminal_width = termion::terminal_size().unwrap().0;
    let terminal_height = termion::terminal_size().unwrap().1;
    let column_names = db.get_column_names(table_name);
    let table_content = db.select_from(table_name);

    let mut out = String::new();

    out += &format!("{}", termion::clear::All);

    // status line
    out += &format!(
        "{}{}{}Table: {}{}",
        Fg(White),
        Bg(Green),
        Goto(1, terminal_height - 2),
        pad(&db.get_name(), terminal_width - 1 - "Table: ".len() as u16),
        Bg(Black),
    );

    // get the max width of each column
    let mut column_widths: Vec<u16> = vec![];
    for column_name in &column_names {
        column_widths.push(column_name.len() as u16);
    }

    for row in &table_content {
        for (idx, column) in row.iter().enumerate() {
            let len = column.to_string().len() as u16;
            if len > column_widths[idx] {
                column_widths[idx] = len;
            }
        }
    }

    // get the position of all columns
    let mut column_pos = vec![];
    let mut pos = 0;
    for column_width in column_widths.iter() {
        column_pos.push(pos);
        pos += column_width + 1;
    }

    // column headers
    let mut line = format!("{}{}{}Row# ", Fg(Red), Bg(Reset), Goto(1, offset_top));
    for (idx, column_name) in column_names.iter().enumerate() {
        line += &pad(column_name, column_widths[idx] + 1);
    }
    line += &" ".repeat(terminal_width as usize - line.len());
    out += &line;
    out += &format!("{}{}", Fg(Reset), Bg(Reset));

    // rows
    for (idx_y, row) in table_content.iter().enumerate() {
        // row id
        out += &format!(
            "{}{}{:4}{}",
            Fg(Red),
            Goto(1, offset_top + idx_y as u16 + 2),
            idx_y + 1,
            Fg(Reset)
        );
        // columns
        for (idx_x, column) in row.iter().enumerate() {
            out += &format!(
                "{}{}",
                Goto(
                    column_pos[idx_x] + offset_left,
                    offset_top + idx_y as u16 + 2
                ),
                column
            );
        }
    }

    // reset color and cursor position
    out += &format!("{}{}{}", Fg(Reset), Bg(Reset), Goto(1, terminal_height - 1));

    // output everything
    write!(stdout, "{}", out).unwrap();
}
