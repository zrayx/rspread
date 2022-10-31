use std::io::{stdout, Write};

use termion::color::{Bg, Fg, Reset};
#[allow(unused_imports)]
use termion::color::{Black, Blue, Cyan, Green, Magenta, Red, White, Yellow};
use termion::cursor::Goto;
use termion::raw::IntoRawMode;
#[allow(unused_imports)]
use termion::raw::RawTerminal;

use rzdb::{Data, Db, Row};

use crate::common::{self, is_cell};
use crate::editor::Editor;
use crate::mode::Mode;
use crate::pos::Pos;

fn expand_table(rows: Vec<Vec<Vec<Data>>>) -> Vec<Row> {
    let mut result = vec![];
    for row in &rows {
        let max_items = row.iter().map(|c| c.len()).max().unwrap_or(0);
        for item_idx in 0..max_items {
            let mut row_data = vec![];
            for col in row {
                if item_idx < col.len() {
                    row_data.push(col[item_idx].clone());
                } else {
                    row_data.push(Data::Empty);
                }
            }
            result.push(Row::from(row_data));
        }
    }
    result
}

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

    let mut column_names_extended = common::get_column_names_extended(db, table_name, cursor.x - 1);
    for (idx, column_name) in &mut column_names_extended.iter_mut().enumerate() {
        if cursor.y != 0 || idx != cursor.x - 1 {
            if let Some(pos) = column_name.find('|') {
                *column_name = column_name[..pos].to_string();
            }
        }
    }

    let table_content = db.select_array(table_name).unwrap();
    let mut table_content = expand_table(table_content);
    while cursor.y > 0 && table_content.len() < cursor.y - 1 {
        table_content.push(Row::new());
    }

    let mut out = String::new();
    let mut offset = Pos::new(0, 0);

    out += &format!("{}{}", termion::cursor::Hide, termion::clear::All);

    // status line
    let line = if cursor.y == 0 {
        format!(
            "Table: {}, Cur: ({},{}), {}, {}",
            table_name,
            cursor.x,
            cursor.y,
            mode,
            column_names_extended[cursor.x - 1]
        )
    } else if is_cell(db, table_name, cursor.x - 1, cursor.y - 1) {
        let cell = table_content[cursor.y - 1].select_at(cursor.x - 1).unwrap();
        let data_type_string = match cell {
            Data::Int(_) => "int",
            Data::Float(_) => "float",
            Data::String(_) => "string",
            Data::Date(_) => "date",
            Data::Time(_) => "time",
            Data::Join(_) => "join",
            Data::Empty => "empty",
        };
        format!(
            "Table: {}, Cur: ({},{}), {}, {}:{}",
            table_name, cursor.x, cursor.y, mode, data_type_string, cell
        )
    } else {
        format!(
            "Table: {}, Cur: ({},{}), {}",
            table_name, cursor.x, cursor.y, mode
        )
    };
    let line = line.chars().take(terminal_width).collect::<String>();
    out += &format!(
        "{}{}{}{}{}",
        Fg(Black),
        Bg(Green),
        Goto(1, terminal_height as u16 - 1),
        pad(&line, terminal_width),
        Bg(Black),
    );

    // get the max width and height of each column
    let mut entry_widths: Vec<usize> = vec![];
    let mut entry_heights: Vec<usize> = vec![];
    for column_name in &column_names_extended {
        entry_widths.push(column_name.chars().count());
    }
    for (row_idx, row) in table_content.iter().enumerate() {
        entry_heights.push(1);
        for (col_idx, column) in row.iter().enumerate() {
            let width = column.no_time_seconds().chars().count();
            if width > entry_widths[col_idx] {
                entry_widths[col_idx] = width;
            }

            let height = column.item_count();
            if height > entry_heights[row_idx] {
                entry_heights[row_idx] = height;
            }
        }
    }

    // length of editor field while editing
    if *mode == Mode::Insert {
        let len = editor.len_utf8();
        if len > entry_widths[cursor.x - 1] {
            entry_widths[cursor.x - 1] = len;
        }
    }

    // get the position of all entries
    // the length of the array is increased by one, as it includes the first not displayed position on the right / bottom
    let mut column_pos = vec![0];
    let mut col_pos = 0;
    for entry_width in entry_widths.iter() {
        col_pos += entry_width + 1;
        column_pos.push(col_pos);
    }

    let mut row_pos = vec![0];
    let mut line_pos = 0;
    for entry_height in entry_heights.iter() {
        line_pos += entry_height;
        row_pos.push(line_pos);
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
        line += &pad(column_name, entry_widths[idx] + 1);
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
        let width = entry_widths[cursor.x - 1];
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
    let num_rows = table_content
        .len()
        .max(if cursor.y > 0 { cursor.y - 1 } else { 0 });
    for idx_y in offset.y..num_rows {
        let cur_row_pos = row_pos[idx_y] - row_pos[offset.y];
        // row id
        out += &format!(
            "{}{}{:4}{}",
            Fg(Red),
            Goto(1, (margin_top + cur_row_pos + 2) as u16),
            idx_y + 1,
            Fg(Reset)
        );
        // columns
        if idx_y < table_content.len() {
            let row = &table_content[idx_y];
            for idx_x in offset.x..num_columns {
                let data = if idx_x < row.len() {
                    let data = row.select_at(idx_x).unwrap_or(Data::Empty);
                    db.from_ids(data.clone())
                        .unwrap_or_else(|_| vec![data; entry_heights[idx_y]])
                } else {
                    vec![Data::Empty; entry_heights[idx_y]]
                };

                for (item_idx, datum) in data.iter().map(|d| d.no_time_seconds()).enumerate() {
                    // render the cursor in inverse
                    if idx_x == cursor.x - 1 && cur_row_pos + item_idx + 1 == cursor.y {
                        out += &format!("{}{}", Fg(Black), Bg(White));
                    } else {
                        out += &format!("{}{}", Fg(White), Bg(Reset));
                    }

                    // check if beyond right edge of window
                    if column_pos[idx_x] - column_pos[offset.x] + margin_left > terminal_width {
                        break;
                    }

                    // don't display data if it is too long
                    let width_left =
                        terminal_width + column_pos[offset.x] - column_pos[idx_x] + 1 - margin_left;
                    let datum = datum.chars().take(width_left).collect::<String>();

                    out += &format!(
                        "{}{}",
                        Goto(
                            (column_pos[idx_x] - column_pos[offset.x] + margin_left) as u16,
                            (margin_top + cur_row_pos + item_idx + 2) as u16
                        ),
                        pad(&datum, entry_widths[idx_x] + 1),
                    );
                    if idx_x == cursor.x - 1 && idx_y + 1 == cursor.y {
                        out += &format!("{}{}", Fg(Reset), Bg(Reset));
                    }
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
                entry_widths[cursor.x - 1],
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
