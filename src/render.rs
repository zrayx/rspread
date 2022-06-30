use rzdb::Db;
use termion::cursor::Goto;
use termion::raw::IntoRawMode;
#[allow(unused_imports)]
use termion::raw::RawTerminal;

use std::io::{stdout, Write};

use crate::cursor::Cursor;
use termion::color::{Bg, Fg, Reset};
#[allow(unused_imports)]
use termion::color::{Black, Blue, Cyan, Green, Magenta, Red, White, Yellow};

pub fn render(db: &Db, table_name: &str, cursor: &Cursor) {
    let mut stdout = stdout().into_raw_mode().unwrap();

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

    out += &format!("{}{}", termion::cursor::Hide, termion::clear::All);

    // status line
    let line = format!("Table: {}, Cur: ({},{})", table_name, cursor.x, cursor.y);
    out += &format!(
        "{}{}{}{}{}",
        Fg(Black),
        Bg(Green),
        Goto(1, terminal_height - 2),
        pad(&line, terminal_width - 1 - line.len() as u16),
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
    if line.len() > terminal_width as usize {
        line = line[..terminal_width as usize].to_string();
    } else {
        line += &" ".repeat(terminal_width as usize - line.len());
    }
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
            if idx_x == cursor.x as usize - 1 && idx_y == cursor.y as usize - 1 {
                out += &format!("{}{}", Fg(Black), Bg(Yellow));
            }
            out += &format!(
                "{}{}",
                Goto(
                    column_pos[idx_x] + offset_left,
                    offset_top + idx_y as u16 + 2
                ),
                column
            );
            if idx_x == cursor.x as usize - 1 && idx_y == cursor.y as usize - 1 {
                out += &format!("{}{}", Fg(Reset), Bg(Reset));
            }
        }
    }

    // reset color and cursor position
    out += &format!("{}{}{}", Fg(Reset), Bg(Reset), Goto(1, terminal_height - 1));

    // output everything
    write!(stdout, "{}", out).unwrap();
    stdout.flush().unwrap();
}

pub fn cleanup() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
