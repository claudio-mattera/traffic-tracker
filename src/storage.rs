use rusqlite::{Connection, Result, NO_PARAMS};
use rusqlite::types::ToSql;

use chrono::prelude::*;

use log::*;


pub fn store_traffic(traffic: i64, database: &str) -> Result<()> {
    info!("Recording traffic in SQLite database \"{}\"", database);

    let connection = Connection::open(database)?;
    create_table(&connection)?;
    insert_data(&connection, traffic)?;

    Ok(())
}


fn fetch_previous_data(connection: &Connection, today: &NaiveDate) -> Result<i64> {
    let yesterday = today.pred();

    debug!("Fetching previous data...");

    let value: i64 = connection.query_row(
        "SELECT traffic FROM traffic WHERE date = ?",
        &[&yesterday as &ToSql],
        |row| row.get(0)
    )?;

    info!("Yesterday's value: {}", value);

    Ok(value)
}

fn insert_data(connection: &Connection, traffic: i64) -> Result<()> {
    debug!("Inserting new data...");

    let now: Date<Utc> = Utc::now().date();
    let today: NaiveDate = now.naive_utc();

    let traffic = match fetch_previous_data(connection, &today) {
        Ok(value) if traffic > value => traffic - value,
        _ => traffic,
    };
    info!("Storing value {}", traffic);

    connection.execute(
        "INSERT INTO traffic (date, traffic) VALUES (?1, ?2)",
        &[&today as &ToSql, &traffic],
    )?;

    Ok(())
}


fn create_table(connection: &Connection) -> Result<()> {
    debug!("Creating table...");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS traffic (date DATETIME, traffic INTEGER, PRIMARY KEY (date))",
        NO_PARAMS,
    )?;

    Ok(())
}
