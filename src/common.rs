use crate::command::Command;
use crate::editor;
use crate::mode::Mode;
use crate::pos::{self, Pos};
use crate::State;
use rzdb::{Data, Db};

pub struct Rect {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
}

pub(crate) fn is_cell(db: &Db, state: &State, x: usize, y: usize) -> bool {
    x < db.get_column_count(&state.table_name).unwrap()
        && y < db.get_row_count(&state.table_name).unwrap()
}

pub(crate) fn get_column_names_extended(db: &Db, table_name: &str, x: usize) -> Vec<String> {
    let mut names = db.get_column_names(table_name).unwrap();
    for idx in names.len()..=x {
        names.push(generate_column_name(db, table_name, idx + 1));
    }
    names
}

pub(crate) fn get_column_name_or_generic(x: usize, db: &Db, table_name: &str) -> String {
    if x <= db.get_column_count(table_name).unwrap() {
        db.get_column_name_at(table_name, x - 1).unwrap()
    } else {
        generate_column_name(db, table_name, x - 1)
    }
}

pub(crate) fn generate_column_name(db: &Db, table_name: &str, x: usize) -> String {
    let column_names = db.get_column_names(table_name).unwrap();
    let mut x = x;
    loop {
        let new_name = format!("Column {}", x);
        if !column_names.contains(&new_name) {
            return new_name;
        }
        x += 1;
    }
}

pub(crate) fn set_error_message(new_message: &str, message: &mut String, mode: &mut Mode) {
    *message = new_message.to_string();
    *mode = Mode::Error;
}

pub(crate) fn set_table(
    new_table_name: &str,
    state: &mut State,
    previous_table_name: &mut String,
    cursor: &mut Pos,
) {
    *previous_table_name = state.table_name.clone();
    state.table_name = new_table_name.to_string();
    *cursor = Pos::new(1, 1);
}

pub(crate) fn load_database(
    state: &State,
    db: &mut Db,
    status_line_message: &mut String,
    mode: &mut Mode,
) {
    let e = Db::load(&state.db_name, &state.db_dir);
    match e {
        Ok(d) => *db = d,
        Err(e) => {
            let mut message = format!("Error loading database: {}", e);
            // return new database if it doesn't exist
            if let Some(std_io_error) = e.downcast_ref::<std::io::Error>() {
                if std_io_error.kind() == std::io::ErrorKind::NotFound {
                    message = format!("Database {} does not exist", state.db_name);
                }
            }
            set_error_message(&message, status_line_message, mode);
        }
    };
}

pub(crate) fn load_table(
    args: &mut std::str::SplitWhitespace,
    state: &mut State,
    previous_table_name: &mut String,
    cursor: &mut pos::Pos,
    db: &mut Db,
    editor: &mut editor::Editor,
) {
    if let Some(arg1) = args.next() {
        // set table_name to new arg
        let new_table_name = arg1.to_string();
        set_table(&new_table_name, state, previous_table_name, cursor);
        if !db.exists(&state.table_name) {
            db.create_table(&state.table_name).unwrap();
        }
        editor.clear();
        *cursor = pos::Pos::new(1, 1);
    }
}

pub(crate) fn drop_table(
    mut args: std::str::SplitWhitespace,
    db: &mut Db,
    state: &mut State,
    previous_table_name: &mut String,
    cursor: &mut pos::Pos,
    mode: &mut Mode,
) -> Result<(), String> {
    if let Some(arg1) = args.next() {
        let name = arg1.to_string();
        if db.drop_table(&name).is_err() {
            return Err(format!("Table {} does not exist", name));
        }
        list_tables(state, previous_table_name, cursor, db, mode);
        Ok(())
    } else {
        Err("No table name given".to_string())
    }
}

pub(crate) fn list_tables(
    state: &mut State,
    previous_table_name: &mut String,
    cursor: &mut pos::Pos,
    db: &mut Db,
    mode: &mut Mode,
) {
    let new_table_name = ".".to_string();
    set_table(&new_table_name, state, previous_table_name, cursor);
    db.create_or_replace_table(&state.table_name).unwrap();
    db.create_column(&state.table_name, "name").unwrap();
    let mut table_names = db.get_table_names();
    table_names.sort();
    for table in table_names {
        if &table != "." {
            db.insert(&state.table_name, vec![&table]).unwrap();
        }
    }
    *mode = Mode::ListTables;
}

pub(crate) fn list_databases(
    state: &mut State,
    previous_table_name: &mut String,
    cursor: &mut pos::Pos,
    db: &mut Db,
    mode: &mut Mode,
) {
    let new_table_name = ".".to_string();
    set_table(&new_table_name, state, previous_table_name, cursor);
    db.create_or_replace_table(&state.table_name).unwrap();
    db.create_column(&state.table_name, "name").unwrap();
    let mut database_names = db.get_database_names().unwrap();
    database_names.sort();
    for database in database_names {
        db.insert(&state.table_name, vec![&database]).unwrap();
    }
    *mode = Mode::ListDatabases;
}

pub(crate) fn extend_table(
    db: &mut Db,
    table_name: &str,
    new_column_count: usize,
    new_row_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_row_count = db.get_row_count(table_name).unwrap();
    let old_column_count = db.get_column_count(table_name).unwrap();
    for idx in old_column_count..new_column_count {
        db.create_column(table_name, &generate_column_name(db, table_name, idx + 1))
            .unwrap();
    }
    for _ in old_row_count..new_row_count {
        let column_count = new_column_count.max(old_column_count);
        db.insert(table_name, vec![""; column_count]).unwrap();
    }
    Ok(())
}

pub(crate) fn editor_enter(
    db: &Db,
    state: &State,
    cursor: &pos::Pos,
    editor: &mut editor::Editor,
    cursor_x: i32,
) {
    let column_count = db.get_column_count(&state.table_name).unwrap();
    let old_text = if cursor.y == 0 {
        if cursor.x > column_count {
            generate_column_name(db, &state.table_name, cursor.x)
        } else {
            db.get_column_name_at(&state.table_name, cursor.x - 1)
                .unwrap()
        }
    } else if is_cell(db, state, cursor.x - 1, cursor.y - 1) {
        db.select_at(&state.table_name, cursor.x - 1, cursor.y - 1)
            .unwrap()
            .no_time_seconds()
    } else {
        "".to_string()
    };
    if cursor_x < 0 {
        let len = old_text.len() as i32;
        if len - cursor_x >= 0 {
            editor.insert_at(&old_text, (len - cursor_x + 1) as usize);
        } else {
            editor.insert_at(&old_text, 0);
        }
    } else {
        editor.insert_at(&old_text, cursor_x as usize);
    }
}

pub(crate) fn editor_exit(
    db: &mut Db,
    state: &State,
    mode: &mut Mode,
    cursor: &mut pos::Pos,
    editor: &mut editor::Editor,
) -> Result<(), Box<dyn std::error::Error>> {
    if !editor.get_line().is_empty() {
        extend_table(db, &state.table_name, cursor.x, cursor.y)?;
    }
    let new_line = editor.get_line();
    editor.clear(); // make sure to clear editor even if we quit with an error
    if cursor.y == 0 {
        // column name
        let old_column_name = get_column_name_or_generic(cursor.x, db, &state.table_name);
        let new_column_name = new_line;
        if old_column_name != new_column_name {
            db.rename_column(&state.table_name, &old_column_name, &new_column_name)?;
        }
    } else if is_cell(db, state, cursor.x - 1, cursor.y - 1) {
        db.set_at(
            &state.table_name,
            cursor.y - 1,
            cursor.x - 1,
            Data::parse(&new_line),
        )?;
    }
    editor.clear();
    *mode = Mode::Normal;
    Ok(())
}

// x/y is 1-indexed; start_y==0 means copy column name
pub(crate) fn yank(
    r: Rect,
    db: &mut Db,
    state: &State,
    clipboard_table_name: &str,
    clipboard: &mut arboard::Clipboard,
) {
    if r.start_y == 0 && r.end_y == 1 {
        let column_name = get_column_name_or_generic(r.start_x, &*db, &state.table_name);
        db.create_or_replace_table(clipboard_table_name).unwrap();
        db.create_column(clipboard_table_name, &column_name)
            .unwrap();
        db.insert(clipboard_table_name, vec![&column_name]).unwrap();
    } else {
        let columns = db.get_column_names(&state.table_name).unwrap();
        let v2: Vec<&str> = columns.iter().map(|s| &**s).collect();

        db.select_into(
            clipboard_table_name,
            &state.table_name,
            &v2[(r.start_x - 1)..(r.end_x - 1)],
            r.start_y - 1,
            r.end_y - 1,
        )
        .unwrap();
    }

    clipboard_to_clipboard(db, clipboard_table_name, clipboard);
}

fn clipboard_to_clipboard(
    db: &mut Db,
    clipboard_table_name: &str,
    clipboard: &mut arboard::Clipboard,
) {
    if let Ok(rows) = db.select_from(clipboard_table_name) {
        let mut clipboard_string = String::new();
        for (i, row) in rows.iter().enumerate() {
            if i > 0 {
                clipboard_string.push('\n');
            }
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    clipboard_string.push('\t');
                }
                clipboard_string.push_str(&cell.to_string());
            }
        }
        clipboard.set_text(clipboard_string).unwrap();
    }
}

#[allow(unused_variables)]
pub(crate) fn paste(
    db: &mut Db,
    state: &State,
    clipboard_table_name: &str,
    cursor: &mut pos::Pos,
    command: &Command,
) {
    let clip_rows_num = db.get_row_count(clipboard_table_name).unwrap();
    let clip_cols_num = db.get_column_count(clipboard_table_name).unwrap();
    let table_rows_num = db.get_row_count(&state.table_name).unwrap();
    let table_cols_num = db.get_column_count(&state.table_name).unwrap();

    let paste_overwrite_cells = *command == Command::PasteReplace;
    let paste_insert_cells = !paste_overwrite_cells;
    let paste_column_header =
        cursor.y == 0 && !paste_insert_cells && clip_cols_num == 1 && clip_rows_num == 1;
    let insert_columns = paste_insert_cells
        && (cursor.y == 0 || (table_cols_num > 1 && clip_cols_num == 1 && clip_rows_num > 1));
    let insert_rows = !paste_overwrite_cells && !paste_column_header && !insert_columns;
    let insert_after = *command == Command::PasteAfter;

    if paste_column_header {
        if let Ok(cell_data) = db.select_at(clipboard_table_name, 0, 0) {
            let new_name = cell_data.to_string();
            let old_name = db
                .get_column_name_at(&state.table_name, cursor.x - 1)
                .unwrap();
            let check_columns = db.get_column_names(&state.table_name).unwrap();
            let new_name = generate_nice_copy_name(&new_name, check_columns);
            db.rename_column(&state.table_name, &old_name, &new_name)
                .unwrap();
        }
        return;
    }

    // calculate paste range, indexs are 0-indexed
    let (start_x, start_y) = if insert_columns {
        (cursor.x - 1 + usize::from(insert_after), 0)
    } else if insert_rows {
        (0, cursor.y - 1 + usize::from(insert_after))
    } else {
        (cursor.x - 1, cursor.y - 1)
    };
    let (end_x, end_y) = (start_x + clip_cols_num, start_y + clip_rows_num);

    // TODO: paste all data, not only one cell
    if paste_overwrite_cells {
        if let Ok(cell_data) = db.select_at(clipboard_table_name, 0, 0) {
            extend_table(db, &state.table_name, end_x, end_y).unwrap();
            db.set_at(&state.table_name, start_y, start_x, cell_data)
                .unwrap();
        }
    } else if insert_rows {
        let table_column_count = db.get_column_count(&state.table_name).unwrap();
        let clipboard_column_count = db.get_column_count(clipboard_table_name).unwrap();
        if cursor.y > table_rows_num {
            extend_table(
                db,
                &state.table_name,
                0,
                cursor.y - 1 + usize::from(insert_after),
            )
            .unwrap();
        }
        extend_table(db, &state.table_name, clipboard_column_count, 0).unwrap();
        extend_table(db, clipboard_table_name, table_column_count, 0).unwrap();
        db.insert_into_at(clipboard_table_name, &state.table_name, start_y)
            .unwrap();
    } else if insert_columns {
        let clipboard_row_count = db.get_row_count(clipboard_table_name).unwrap();
        let table_row_count = db.get_row_count(&state.table_name).unwrap();
        extend_table(db, &state.table_name, 0, clipboard_row_count).unwrap();
        extend_table(db, clipboard_table_name, 0, table_row_count).unwrap();
        // make sure column names are unique
        let table_columns = db.get_column_names(&state.table_name).unwrap();
        let mut clipboard_columns = db.get_column_names(clipboard_table_name).unwrap();
        let old_clipboard_columns = clipboard_columns.clone();
        let mut new_column_names = vec![];
        for column_name in &clipboard_columns.clone() {
            if table_columns.contains(column_name) {
                let mut check_columns = old_clipboard_columns.clone();
                check_columns.append(&mut table_columns.clone());
                check_columns.append(&mut new_column_names.clone());
                let new_column_name = generate_nice_copy_name(column_name, check_columns);
                new_column_names.push(new_column_name);
            } else {
                new_column_names.push(column_name.to_string());
            }
        }
        clipboard_columns = new_column_names;

        for (i, column_name) in clipboard_columns.iter_mut().enumerate() {
            if column_name != &old_clipboard_columns[i] {
                db.rename_column(clipboard_table_name, &old_clipboard_columns[i], column_name)
                    .unwrap();
            }
        }

        db.insert_columns_at(clipboard_table_name, &state.table_name, start_x)
            .unwrap();

        if insert_after {
            if insert_rows {
                cursor.y += 1;
            } else if insert_columns {
                cursor.x += 1;
            }
        }
    } else {
        unreachable!("unreachable() reached in paste()");
    }
}

pub fn generate_nice_copy_name(from_name: &str, from_vec: Vec<String>) -> String {
    let check_for_num = |s: &str| -> (usize, u64) {
        let len_utf8 = s.chars().count();
        let mut index = len_utf8 - 1;
        let s_nth = |n| s.chars().nth(n).unwrap();
        if s.is_empty() {
            return (len_utf8, 1);
        }
        if s_nth(index) != ')' {
            return (len_utf8, 1);
        }
        index -= 1;
        let mut number = 0;
        while index > 0 && s_nth(index) >= '0' && s_nth(index) <= '9' {
            number = number * 10 + (s_nth(index) as u64 - '0' as u64);
            index -= 1;
        }
        if s_nth(index) != '(' {
            return (len_utf8, 1);
        }
        (index, number)
    };

    let (index, mut copy_name_index) = check_for_num(from_name);
    let from_name = from_name.chars().take(index).collect::<String>();
    loop {
        let copy_name = format!("{}({})", from_name, copy_name_index + 1);
        if !from_vec.contains(&copy_name) {
            return copy_name;
        }
        copy_name_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_nice_copy_name() {
        let from_name = "test";
        let from_vec = vec![
            "test(1)".to_string(),
            "test(2)".to_string(),
            "test(3)".to_string(),
        ];
        let copy_name = generate_nice_copy_name(from_name, from_vec);
        assert_eq!(copy_name, "test(4)");

        let from_name = "test(2)";
        let from_vec = vec![
            "test(1)".to_string(),
            "test(2)".to_string(),
            "test(3)".to_string(),
        ];
        let copy_name = generate_nice_copy_name(from_name, from_vec);
        assert_eq!(copy_name, "test(4)");

        let from_name = "test2)";
        let from_vec = vec![
            "test(1)".to_string(),
            "test(2)".to_string(),
            "test(3)".to_string(),
        ];
        let copy_name = generate_nice_copy_name(from_name, from_vec);
        assert_eq!(copy_name, "test2)(2)");

        let from_name = "test2";
        let from_vec = vec![
            "test(1)".to_string(),
            "test(2)".to_string(),
            "test(3)".to_string(),
        ];
        let copy_name = generate_nice_copy_name(from_name, from_vec);
        assert_eq!(copy_name, "test2(2)");
    }
}
