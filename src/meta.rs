use rzdb::Db;

const _RECENT_TABLES: &str = "recent_tables";

pub(crate) fn insert_recent_table(
    mut meta_db: Db,
    db_dir: &String,
    db_name: &String,
    table_name: &String,
) {
    // create recents table if it doesn't exist
    if !meta_db.exists(_RECENT_TABLES) {
        meta_db.create_table(_RECENT_TABLES).unwrap();
        meta_db.create_column(_RECENT_TABLES, "db_dir").unwrap();
        meta_db.create_column(_RECENT_TABLES, "db_name").unwrap();
        meta_db.create_column(_RECENT_TABLES, "table_name").unwrap();
    }
    // create recent table entry if it doesn't exist
    let r = meta_db
        .select_columns(_RECENT_TABLES, &["db_dir", "db_name", "table_name"])
        .unwrap();
    let exists = r.iter().any(|row| {
        row.select_at(0).unwrap().to_string() == *db_dir
            && row.select_at(1).unwrap().to_string() == *db_name
            && row.select_at(2).unwrap().to_string() == *table_name
    });
    if !exists {
        meta_db
            .insert(_RECENT_TABLES, vec![db_dir, db_name, table_name])
            .unwrap();
        meta_db.save().unwrap();
    }
}
