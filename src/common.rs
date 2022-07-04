use crate::mode::Mode;
use crate::pos::Pos;
use rzdb::Db;

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
