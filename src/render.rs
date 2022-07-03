use std::io::{stdout, Write};

use termion::color::{Bg, Fg, Reset};
#[allow(unused_imports)]
use termion::color::{Black, Blue, Cyan, Green, Magenta, Red, White, Yellow};
use termion::cursor::Goto;
use termion::raw::IntoRawMode;
#[allow(unused_imports)]
use termion::raw::RawTerminal;

use rzdb::Db;

use crate::common;
use crate::cursor::Cursor;
use crate::editor::Editor;
use crate::mode::Mode;

pub fn render(db: &Db, table_name: &str, cursor: &Cursor, mode: &Mode, editor: &Editor) {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let pad = |s: &str, width: usize| {
        let mut s = s.to_string();
        while (s.len()) < width {
            s.push(' ');
        }
        s
    };

    let offset_left = 6; // room for row id
    let offset_top: usize = 0; // room for status line+column headers
    let terminal_width = termion::terminal_size().unwrap().0 as usize;
    let terminal_height = termion::terminal_size().unwrap().1 as usize;
    let column_names_extended = common::get_column_names_extended(db, table_name, cursor.x - 1);
    let table_content = db.select_from(table_name);

    let mut out = String::new();

    out += &format!("{}{}", termion::cursor::Hide, termion::clear::All);

    // status line
    let line = format!(
        "Table: {}, Cur: ({},{}), {}",
        table_name, cursor.x, cursor.y, mode
    );
    out += &format!(
        "{}{}{}{}{}",
        Fg(Black),
        Bg(Green),
        Goto(1, terminal_height as u16),
        pad(&line, terminal_width - 1 - line.len()),
        Bg(Black),
    );

    // get the max width of each column
    let mut column_widths: Vec<usize> = vec![];
    for column_name in &column_names_extended {
        column_widths.push(column_name.len());
    }

    for row in &table_content {
        for (idx, column) in row.iter().enumerate() {
            let len = column.to_string().len();
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
    let mut line = "Row# ".to_string();
    let num_columns = column_names_extended.len().max(cursor.x - 1);
    for idx in 0..num_columns {
        let column_name = &column_names_extended[idx];
        line += &pad(column_name, column_widths[idx] + 1);
    }
    if line.len() > terminal_width {
        line = line[..terminal_width].to_string();
    } else {
        line += &" ".repeat(terminal_width - line.len());
    }
    out += &format!(
        "{}{}{}{}{}{}",
        Goto(1, offset_top as u16),
        Fg(Red),
        Bg(Reset),
        line,
        Fg(Reset),
        Bg(Reset),
    );
    if cursor.y == 0 {
        let x = column_pos[cursor.x - 1] + offset_left;
        let width = column_widths[cursor.x - 1];
        out += &format!(
            "{}{}{}{}{}{}",
            Goto(x as u16, offset_top as u16 + 1),
            Fg(Black),
            Bg(Red),
            pad(&column_names_extended[cursor.x - 1], width),
            Fg(Reset),
            Bg(Reset),
        );
    }

    // rows
    let num_rows = table_content.len().max(cursor.y).min(terminal_height);
    for idx_y in 0..num_rows {
        // row id
        out += &format!(
            "{}{}{:4}{}",
            Fg(Red),
            Goto(1, (offset_top + idx_y + 2) as u16),
            idx_y + 1,
            Fg(Reset)
        );
        // columns
        if idx_y < table_content.len() {
            let row = &table_content[idx_y];
            for (idx_x, column) in row.iter().enumerate() {
                // render the cursor in inverse
                if idx_x == cursor.x - 1 && idx_y + 1 == cursor.y {
                    out += &format!("{}{}", Fg(Black), Bg(White));
                }
                let data = column.to_string();
                out += &format!(
                    "{}{}",
                    Goto(
                        (column_pos[idx_x] + offset_left) as u16,
                        (offset_top + idx_y + 2) as u16
                    ),
                    pad(&data, column_widths[idx_x] + 1),
                );
                if idx_x == cursor.x - 1 && idx_y + 1 == cursor.y {
                    out += &format!("{}{}", Fg(Reset), Bg(Reset));
                }
            }
        }
    }

    // render editor cell / cursor if outside existing cells
    if *mode == Mode::Insert
        || cursor.y > table_content.len()
        || (cursor.y > 0 && cursor.x > table_content[0].len())
    {
        let (line, bg) = if *mode == Mode::Insert {
            (
                pad(&editor.line, column_widths[cursor.x - 1]),
                format!("{}", Bg(Yellow)),
            )
        } else {
            (
                pad("", column_widths[cursor.x - 1]),
                format!("{}", Bg(White)),
            )
        };

        out += &bg;
        out += &format!(
            "{}{}{}{}{}",
            Fg(Black),
            Goto(
                (column_pos[cursor.x - 1] + offset_left) as u16,
                (offset_top + cursor.y + 1) as u16
            ),
            line,
            Fg(Reset),
            Bg(Reset),
        );

        // render cursor of editor
        if *mode == Mode::Insert {
            let ch = if editor.cur_x >= editor.line.len() {
                " ".to_string()
            } else {
                editor.line.chars().nth(editor.cur_x).unwrap().to_string()
            };
            out += &format!(
                "{}{}{}{}{}{}",
                Bg(Blue),
                Fg(Black),
                Goto(
                    (column_pos[cursor.x - 1] + offset_left + editor.cur_x) as u16,
                    (offset_top + cursor.y + 1) as u16
                ),
                ch,
                Bg(Reset),
                Fg(Reset),
            );
        }
    }

    // reset color and cursor position
    out += &format!(
        "{}{}{}",
        Fg(Reset),
        Bg(Reset),
        Goto(1, terminal_height as u16 - 1)
    );

    // output everything
    write!(stdout, "{}", out).unwrap();
    stdout.flush().unwrap();
}

pub fn cleanup() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
