Traffic Tracker
====

An application to fetch traffic statistics from GSM router Huawei E5172As-22.

When using a GSM router, often the connection is capped to a monthly total traffic, depending on the specific plan available on the SIM card.
This application logs in to the router's web interface, retrieves the current total traffic and stores it in a SQLite database.
On the last day of the month, it also clears the total traffic on the router.


Configuration
----

The following configuration file contains the settings to access the router and the database name to store the traffic statistics.

    base_url = "http://192.168.1.1/"
    username = "admin"
    password = "...encoded_password..."
    database = "traffic.db"

The encoded password can be obtained using a network analyser and manually log on to the router.


Traffic Database
----

The resulting database contains a single table with three fields: `date`, `traffic` and `cumulative_traffic`.
`traffic` contains the daily traffic, while `cumulative_traffic` contains the cumulative traffic since the beginning of the month.
Traffic values are in bytes.
