//! The rows a builder reads, handed in as values.
//!
//! The Ruby builders that make a prompt (a narration context, a ledger, a
//! memory, a floor plan, a System One request) read many tables at once. Here
//! they read a [`Records`]: every row of every table, as the Ruby engine's
//! `lib/engine_vectors/records.rb` writes them down, `{ table => [row, ...] }`
//! with rows in id order. Nothing here is a database; a query is a filter over
//! a list, in the order the Ruby query would return its rows.

use serde_json::{Map, Value};
use std::collections::HashMap;

/// One row: its columns by name.
pub type Row = Map<String, Value>;

#[derive(Clone, Debug, Default)]
pub struct Records {
    tables: HashMap<String, Vec<Row>>,
}

static EMPTY: Vec<Row> = Vec::new();

impl Records {
    /// Reads a dump: an object of tables, each an array of row objects.
    pub fn from_json(dump: &Value) -> Records {
        let tables = dump
            .as_object()
            .expect("records are an object of tables")
            .iter()
            .map(|(table, rows)| {
                let rows = rows
                    .as_array()
                    .unwrap_or_else(|| panic!("{table} is not a list of rows"))
                    .iter()
                    .map(|row| {
                        row.as_object()
                            .unwrap_or_else(|| panic!("a row of {table} is not an object"))
                            .clone()
                    })
                    .collect();
                (table.clone(), rows)
            })
            .collect();
        Records { tables }
    }

    /// Every row of a table, in id order; none for a table with no rows.
    pub fn table(&self, name: &str) -> &[Row] {
        self.tables.get(name).unwrap_or(&EMPTY)
    }

    /// The rows of a table that satisfy `keep`, in id order.
    pub fn select<'a>(&'a self, name: &str, keep: impl Fn(&Row) -> bool) -> Vec<&'a Row> {
        self.table(name).iter().filter(|row| keep(row)).collect()
    }

    /// The first row of a table that satisfies `keep`.
    pub fn first(&self, name: &str, keep: impl Fn(&Row) -> bool) -> Option<&Row> {
        self.table(name).iter().find(|row| keep(row))
    }

    /// The row of a table with this id.
    pub fn find(&self, name: &str, id: i64) -> Option<&Row> {
        self.first(name, |row| int(row, "id") == Some(id))
    }
}

/// An integer column, or none where it is null.
pub fn int(row: &Row, column: &str) -> Option<i64> {
    row.get(column).and_then(Value::as_i64)
}

/// A string column, or none where it is null.
pub fn text<'a>(row: &'a Row, column: &str) -> Option<&'a str> {
    row.get(column).and_then(Value::as_str)
}

/// A string column read as Ruby's `to_s` reads it: null is empty.
pub fn string<'a>(row: &'a Row, column: &str) -> &'a str {
    text(row, column).unwrap_or("")
}

/// A boolean column; null is false.
pub fn flag(row: &Row, column: &str) -> bool {
    row.get(column).and_then(Value::as_bool).unwrap_or(false)
}

/// The row's id.
pub fn id(row: &Row) -> i64 {
    int(row, "id").expect("a row with an id")
}
