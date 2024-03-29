use crate::State;
use rzdb::{Condition, ConditionType, Data, Db};

const _RECENT_TABLES: &str = "recent_tables";

pub(crate) fn insert_recent_table(
    meta_db: &mut Db,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    // create recents table if it doesn't exist
    if !meta_db.exists(_RECENT_TABLES) {
        meta_db.create_table(_RECENT_TABLES)?;
        meta_db.create_column(_RECENT_TABLES, "db_dir")?;
        meta_db.create_column(_RECENT_TABLES, "db_name")?;
        meta_db.create_column(_RECENT_TABLES, "table_name")?;
    }
    // create recent table entry if it doesn't exist
    meta_db.delete_where(
        _RECENT_TABLES,
        &[
            Condition::new(
                "db_dir",
                Data::String(state.db_dir.clone()),
                ConditionType::Equal,
            ),
            Condition::new(
                "db_name",
                Data::String(state.db_name.clone()),
                ConditionType::Equal,
            ),
            Condition::new(
                "table_name",
                Data::String(state.table_name.clone()),
                ConditionType::Equal,
            ),
        ],
    )?;
    meta_db.insert_at(
        _RECENT_TABLES,
        vec![&state.db_dir, &state.db_name, &state.table_name],
        0,
    )?;
    meta_db.save()?;
    Ok(())
}
