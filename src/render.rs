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
use crate::editor::Editor;
use crate::mode::Mode;
use crate::pos::Pos;

pub fn render(
    db: &Db,
    table_name: &str,
    cursor: &Pos,
    mode: &Mode,
    editor: &Editor,
    message: &str,
) {
    let mut stdout = stdout().into_raw_mode().unwrap();

    let pad = |s: &str, width: usize| {
        let mut s = s.to_string();
        while (s.chars().count()) < width {
            s.push(' ');
        }
        s
    };

    let margin_left = 6; // room for row id
    let margin_top: usize = 0; // nothing for now
    let margin_bottom: usize = 2; // room for status line+command line
    let terminal_width = termion::terminal_size().unwrap().0 as usize;
    let terminal_height = termion::terminal_size().unwrap().1 as usize;
    let table_content = db.select_from(table_name).unwrap();
    let mut column_names_extended = common::get_column_names_extended(db, table_name, cursor.x - 1);
    for (idx, column_name) in &mut column_names_extended.iter_mut().enumerate() {
        if cursor.y != 0 || idx != cursor.x - 1 {
            if let Some(pos) = column_name.find('|') {
                *column_name = column_name[..pos].to_string();
            }
        }
    }

    let mut out = String::new();
    let mut offset = Pos::new(0, 0);

    out += &format!("{}{}", termion::cursor::Hide, termion::clear::All);

    // status line
    let line = format!(
        "Table: {}, Cur: ({},{}), {}",
        table_name, cursor.x, cursor.y, mode
    );
    let line = line.chars().take(terminal_width).collect::<String>();
    out += &format!(
        "{}{}{}{}{}",
        Fg(Black),
        Bg(Green),
        Goto(1, terminal_height as u16 - 1),
        pad(&line, terminal_width),
        Bg(Black),
    );

    // get the max width of each column
    let mut column_widths: Vec<usize> = vec![];
    for column_name in &column_names_extended {
        column_widths.push(column_name.chars().count());
    }
    for row in &table_content {
        for (idx, column) in row.iter().enumerate() {
            let len = column.no_time_seconds().chars().count();
            if len > column_widths[idx] {
                column_widths[idx] = len;
            }
        }
    }

    // length of editor field while editing
    if *mode == Mode::Insert {
        let len = editor.len_utf8();
        if len > column_widths[cursor.x - 1] {
            column_widths[cursor.x - 1] = len;
        }
    }

    // get the position of all columns
    // the length of the array is the number of columns + 1, as it includes the first not displayed position on the right
    let mut column_pos = vec![0];
    let mut pos = 0;
    for column_width in column_widths.iter() {
        pos += column_width + 1;
        column_pos.push(pos);
    }

    // move cursor into view if it would be outside
    loop {
        let rightmost = margin_left + column_pos[cursor.x] - column_pos[offset.x];
        if rightmost <= terminal_width || offset.x == cursor.x - 1 {
            break;
        }
        offset.x += 1;
    }
    if cursor.y > terminal_height - margin_top - margin_bottom - 2 {
        offset.y = cursor.y - (terminal_height - margin_top - margin_bottom - 2);
    }

    // column headers
    let mut line = "Row# ".to_string();
    let num_columns = column_names_extended.len().max(cursor.x - 1);
    for idx in offset.x..num_columns {
        let column_name = &column_names_extended[idx];
        line += &pad(column_name, column_widths[idx] + 1);
    }
    if line.chars().count() > terminal_width - margin_left {
        line = line.chars().take(terminal_width).collect::<String>();
    } else {
        line += &" ".repeat(terminal_width - line.chars().count());
    }
    out += &format!(
        "{}{}{}{}{}{}",
        Goto(1, margin_top as u16),
        Fg(Red),
        Bg(Reset),
        line,
        Fg(Reset),
        Bg(Reset),
    );
    // render cursor in column header
    if cursor.y == 0 {
        let x = column_pos[cursor.x - 1] - column_pos[offset.x] + margin_left;
        let width = column_widths[cursor.x - 1];
        out += &format!(
            "{}{}{}{}{}{}",
            Goto(x as u16, margin_top as u16 + 1),
            Fg(Black),
            Bg(Red),
            pad(&column_names_extended[cursor.x - 1], width),
            Fg(Reset),
            Bg(Reset),
        );
    }

    // rows
    let num_rows = table_content.len().max(cursor.y).min(terminal_height);
    let last_row = num_rows.min(terminal_height - margin_top - margin_bottom - 1);
    for idx_y in 0..last_row {
        // row id
        out += &format!(
            "{}{}{:4}{}",
            Fg(Red),
            Goto(1, (margin_top + idx_y + 2) as u16),
            idx_y + offset.y + 1,
            Fg(Reset)
        );
        // columns
        if idx_y + offset.y < table_content.len() {
            let row = &table_content[idx_y + offset.y];
            for idx_x in offset.x..num_columns {
                let data = if idx_x < row.len() {
                    row.select_at(idx_x).unwrap().no_time_seconds()
                } else {
                    "".to_string()
                };

                // render the cursor in inverse
                if idx_x == cursor.x - 1 && idx_y + offset.y + 1 == cursor.y {
                    out += &format!("{}{}", Fg(Black), Bg(White));
                }

                // check if beyond right edge of window
                if column_pos[idx_x] - column_pos[offset.x] + margin_left > terminal_width {
                    break;
                }

                // don't display data if it is too long
                let width_left =
                    terminal_width + column_pos[offset.x] - column_pos[idx_x] + 1 - margin_left;
                let data = data.chars().take(width_left).collect::<String>();

                out += &format!(
                    "{}{}",
                    Goto(
                        (column_pos[idx_x] - column_pos[offset.x] + margin_left) as u16,
                        (margin_top + idx_y + 2) as u16
                    ),
                    pad(&data, column_widths[idx_x] + 1),
                );
                if idx_x == cursor.x - 1 && idx_y + offset.y + 1 == cursor.y {
                    out += &format!("{}{}", Fg(Reset), Bg(Reset));
                }
            }
        }
    }

    // render editor cell / cursor if outside existing cells
    if *mode == Mode::Insert
        || *mode == Mode::Command
        || cursor.y > table_content.len()
        || (cursor.y > 0 && cursor.x > table_content[0].len())
    {
        let (x_pos, y_pos, cursor_len, prefix) = if *mode == Mode::Command {
            (1, terminal_height as u16, terminal_width, ":")
        } else {
            (
                (column_pos[cursor.x - 1] + margin_left - column_pos[offset.x]) as u16,
                (margin_top + cursor.y - offset.y + 1) as u16,
                column_widths[cursor.x - 1],
                "",
            )
        };

        let (line, bg) = if *mode == Mode::Insert || *mode == Mode::Command {
            (
                format!(
                    "{}{}",
                    prefix,
                    pad(&editor.line, cursor_len - prefix.chars().count())
                ),
                format!("{}", Bg(Yellow)),
            )
        } else {
            (pad("", cursor_len), format!("{}", Bg(White)))
        };

        out += &bg;
        out += &format!(
            "{}{}{}{}{}",
            Fg(Black),
            Goto(x_pos, y_pos),
            line,
            Fg(Reset),
            Bg(Reset),
        );

        // render cursor of editor
        if *mode == Mode::Insert || *mode == Mode::Command {
            let ch = if editor.cur_x >= editor.line.chars().count() {
                " ".to_string()
            } else {
                editor.line.chars().nth(editor.cur_x).unwrap().to_string()
            };
            out += &format!(
                "{}{}{}{}{}{}",
                Bg(Blue),
                Fg(Black),
                Goto(
                    x_pos + editor.cur_x as u16 + prefix.chars().count() as u16,
                    y_pos
                ),
                ch,
                Bg(Reset),
                Fg(Reset),
            );
        }
    }

    if *mode == Mode::Error {
        out += &format!(
            "{}{}{}{}{}{}",
            Bg(Red),
            Fg(Black),
            Goto(1, terminal_height as u16),
            pad(message, terminal_width),
            Bg(Reset),
            Fg(Reset),
        );
    }

    // black border horizontally around each cell
    //out += &format!("{}", Bg(Black));
    let last_row = table_content
        .len()
        .min(terminal_height - margin_top - margin_bottom - 1);
    let last_column = if table_content.is_empty() {
        0
    } else {
        table_content[0].len()
    };
    for idx_y in 2..=(last_row + 1) {
        for idx_x in (offset.x + 1)..last_column {
            let x = column_pos[idx_x] - column_pos[offset.x] + margin_left - 1;
            if x > terminal_width {
                break;
            }
            let col_pos = Pos {
                x: idx_x,
                y: idx_y + offset.y - 1,
            };
            if *mode == Mode::Normal || col_pos != *cursor {
                out += &format!("{}·", Goto(x as u16, idx_y as u16),);
            }
        }
    }

    // reset color and cursor position
    out += &format!(
        "{}{}{}",
        Fg(Reset),
        Bg(Reset),
        Goto(1, terminal_height as u16 - 1),
    );

    // output everything
    write!(stdout, "{}", out).unwrap();
    stdout.flush().unwrap();
}

pub fn cleanup() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let (terminal_width, terminal_height) = termion::terminal_size().unwrap();
    let spaces = " ".repeat(terminal_width as usize);
    // reset color and cursor position
    write!(
        stdout,
        "{}{}{}{}{}{}{}{}",
        Fg(Reset),
        Bg(Reset),
        Goto(1, terminal_height - 2),
        spaces,
        Goto(1, terminal_height - 1),
        spaces,
        Goto(1, terminal_height),
        termion::cursor::Show
    )
    .unwrap();
    stdout.flush().unwrap();
}
