use rusqlite::types::ToSql;
use rusqlite::{Connection, Result, NO_PARAMS};

use chrono::prelude::*;

use log::*;

use crate::types::Bytes;

pub fn store_traffic(traffic: i64, database: &str) -> Result<()> {

    let connection = Connection::open(database)?;
    create_table(&connection)?;
    insert_data(&connection, traffic)?;

    Ok(())
}

fn fetch_previous_data(connection: &Connection, today: NaiveDate) -> Result<i64> {
    let yesterday = today.pred();

    debug!("Fetching previous data");

    let value: i64 = connection.query_row(
        "SELECT cumulative_traffic FROM traffic WHERE date = ?",
        &[&yesterday as &ToSql],
        |row| row.get(0),
    )?;

    info!("Yesterday's value: {}", value);

    Ok(value)
}

fn insert_data(connection: &Connection, today_cumulative_traffic: i64) -> Result<()> {
    debug!("Inserting new value {}", Bytes::new(today_cumulative_traffic));

    let now: Date<Utc> = Utc::now().date();
    let today: NaiveDate = now.naive_utc();

    let today_instantaneous_traffic = match fetch_previous_data(connection, today) {
        Ok(yesterday_cumulative_traffic) if today_cumulative_traffic > yesterday_cumulative_traffic => {
            info!(
                "Subtracting value from previous day: {}",
                Bytes::new(yesterday_cumulative_traffic),
            );
            today_cumulative_traffic - yesterday_cumulative_traffic
        }
        Ok(yesterday_cumulative_traffic) => {
            info!(
                "Ignoring value from previous day: {}",
                Bytes::new(yesterday_cumulative_traffic),
            );
            today_cumulative_traffic
        }
        _ => {
            info!("Ignoring missing value from previous day");
            today_cumulative_traffic
        }
    };
    info!("Storing values {}, {}", today_instantaneous_traffic, today_cumulative_traffic);

    connection.execute(
        "INSERT INTO traffic (date, traffic, cumulative_traffic) VALUES (?1, ?2, ?3)",
        &[&today as &ToSql, &today_instantaneous_traffic, &today_cumulative_traffic],
    )?;

    Ok(())
}

fn create_table(connection: &Connection) -> Result<()> {
    debug!("Creating table");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS traffic (
            date DATETIME,
            traffic INTEGER,
            cumulative_traffic INTEGER,
            PRIMARY KEY (date)
        )",
        NO_PARAMS,
    )?;

    Ok(())
}
