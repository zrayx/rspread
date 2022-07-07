use crate::editor;
use crate::mode::Mode;
use crate::pos::{self, Pos};
use rzdb::{Data, Db};

pub(crate) fn is_cell(db: &Db, table_name: &str, x: usize, y: usize) -> bool {
    x < db.get_column_count(table_name).unwrap() && y < db.get_row_count(table_name).unwrap()
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

pub(crate) fn set_table(new_table_name: &str, table_name: &mut String, cursor: &mut Pos) {
    *table_name = new_table_name.to_string();
    *cursor = Pos::new(1, 1);
}

pub(crate) fn load_table(
    args: &mut std::str::SplitWhitespace,
    table_name: &mut String,
    cursor: &mut pos::Pos,
    db: &mut Db,
    editor: &mut editor::Editor,
) {
    if let Some(arg1) = args.next() {
        // set table_name to new arg
        let new_table_name = arg1.to_string();
        set_table(&new_table_name, table_name, cursor);
        if !db.exists(&*table_name) {
            db.create_table(&*table_name).unwrap();
        }
        editor.clear();
        *cursor = pos::Pos::new(1, 1);
    }
}

pub(crate) fn drop_table(
    mut args: std::str::SplitWhitespace,
    db: &mut Db,
    table_name: &mut String,
    cursor: &mut pos::Pos,
    mode: &mut Mode,
) {
    if let Some(arg1) = args.next() {
        let name = arg1.to_string();
        db.drop_table(&name).unwrap();
        list_tables(table_name, cursor, db, mode);
    }
}

pub(crate) fn list_tables(
    table_name: &mut String,
    cursor: &mut pos::Pos,
    db: &mut Db,
    mode: &mut Mode,
) {
    let new_table_name = ".".to_string();
    set_table(&new_table_name, table_name, cursor);
    db.create_or_replace_table(&*table_name).unwrap();
    db.create_column(&*table_name, "name").unwrap();
    let mut table_names = db.get_table_names();
    table_names.sort();
    for table in table_names {
        if &table != "." {
            db.insert(&*table_name, vec![&table]).unwrap();
        }
    }
    *mode = Mode::ListTables;
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
    table_name: &str,
    cursor: &pos::Pos,
    editor: &mut editor::Editor,
    cursor_x: i32,
) {
    let column_count = db.get_column_count(table_name).unwrap();
    let old_text = if cursor.y == 0 {
        if cursor.x > column_count {
            generate_column_name(db, table_name, cursor.x)
        } else {
            db.get_column_name_at(table_name, cursor.x - 1).unwrap()
        }
    } else if is_cell(db, table_name, cursor.x - 1, cursor.y - 1) {
        db.select_at(table_name, cursor.x - 1, cursor.y - 1)
            .unwrap()
            .to_string()
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
    table_name: &str,
    mode: &mut Mode,
    cursor: &mut pos::Pos,
    editor: &mut editor::Editor,
) -> Result<(), Box<dyn std::error::Error>> {
    if !editor.get_line().is_empty() {
        extend_table(db, table_name, cursor.x, cursor.y)?;
    }
    if cursor.y == 0 {
        // column name
        let old_column_name = get_column_name_or_generic(cursor.x, db, table_name);
        let new_column_name = editor.get_line();
        db.rename_column(table_name, &old_column_name, &new_column_name)?;
    } else if is_cell(db, table_name, cursor.x - 1, cursor.y - 1) {
        db.set_at(
            table_name,
            cursor.y - 1,
            cursor.x - 1,
            Data::parse(&editor.line),
        )?;
    }
    editor.clear();
    *mode = Mode::Normal;
    Ok(())
}

// x/y is 1-indexed; start_y==0 means copy column name
pub(crate) fn yank(
    start_x: usize,
    end_x: usize,
    start_y: usize,
    end_y: usize,
    db: &mut Db,
    table_name: &str,
    clipboard_table_name: &str,
) {
    if start_y == 0 && end_y == 1 {
        let column_name = get_column_name_or_generic(start_x, &*db, table_name);
        db.create_or_replace_table(clipboard_table_name).unwrap();
        db.create_column(clipboard_table_name, &column_name)
            .unwrap();
        db.insert(clipboard_table_name, vec![&column_name]).unwrap();
    } else {
        let columns = db.get_column_names(table_name).unwrap();
        let v2: Vec<&str> = columns.iter().map(|s| &**s).collect();

        db.select_into(
            clipboard_table_name,
            table_name,
            &v2[(start_x - 1)..(end_x - 1)],
            start_y - 1,
            end_y - 1,
        )
        .unwrap();
    }
}
#[allow(unused_variables)]
pub(crate) fn paste(
    start_x: usize,
    end_x: usize,
    start_y: usize,
    end_y: usize,
    db: &mut Db,
    table_name: &str,
    clipboard_table_name: &str,
) {
    let paste_column_header = start_y == 0 && end_y == 1;
    let paste_overwrite_cells = start_x != 0 && start_y != 0;
    let paste_rows = start_x == 0;
    let paste_columns = start_y == 0 && end_y != 1;
    // TODO: paste all data, not only one cell
    if paste_column_header || paste_overwrite_cells {
        if let Ok(cell_data) = db.select_at(clipboard_table_name, 0, 0) {
            if paste_overwrite_cells {
                extend_table(db, table_name, end_x - 1, end_y - 1).unwrap();
                db.set_at(table_name, start_y - 1, start_x - 1, cell_data)
                    .unwrap();
            } else {
                let new_name = cell_data.to_string();
                let old_name = db.get_column_name_at(table_name, start_x - 1).unwrap();
                db.rename_column(table_name, &old_name, &new_name).unwrap();
            }
        }
    } else {
        let clipboard_column_count = db.get_column_count(clipboard_table_name).unwrap();
        let clipboard_row_count = db.get_row_count(clipboard_table_name).unwrap();
        let table_column_count = db.get_column_count(table_name).unwrap();
        let table_row_count = db.get_row_count(table_name).unwrap();
        if paste_rows {
            extend_table(db, table_name, table_column_count, 0).unwrap();
            db.insert_into_at(clipboard_table_name, table_name, start_y - 1)
                .unwrap();
        } else if paste_columns {
            // TODO
            let _y = "";
        } else {
            unreachable!("unreachable() reached in paste()");
        }
    }
}
