use rusqlite::{params, Connection};

use crate::domain::analytics::{Category, CreateCategoryInput};

pub(crate) fn list_category_entries(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_id, name, root_type, color_hex
             FROM categories
             ORDER BY id ASC",
        )
        .map_err(|e| format!("failed to prepare categories query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                root_type: row.get(3)?,
                color_hex: row.get(4)?,
            })
        })
        .map_err(|e| format!("failed to read categories: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to decode category row: {e}"))?);
    }

    Ok(result)
}

pub(crate) fn create_category_entry(
    conn: &Connection,
    input: CreateCategoryInput,
) -> Result<i64, String> {
    let root_type: String = conn
        .query_row(
            "SELECT root_type FROM categories WHERE id = ?1",
            [input.parent_id],
            |row| row.get(0),
        )
        .map_err(|_| "parent category not found".to_string())?;

    conn.execute(
        "INSERT INTO categories (parent_id, name, color_hex, root_type) VALUES (?1, ?2, ?3, ?4)",
        params![
            input.parent_id,
            input.name.trim(),
            input
                .color_hex
                .unwrap_or_else(|| if root_type == "LEARN" { "#4ade80" } else { "#fb923c" }.to_string()),
            root_type
        ],
    )
    .map_err(|e| format!("failed to create category: {e}"))?;

    Ok(conn.last_insert_rowid())
}
