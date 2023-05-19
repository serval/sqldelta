use std::thread::spawn;

use sqldelta::UpdateNotifications;

fn main() {
    let conn = sqlite::open(":memory:").unwrap();
    conn.execute(
        "CREATE TABLE contacts (
        contact_id INTEGER PRIMARY KEY,
        first_name TEXT NOT NULL,
        last_name TEXT NOT NULL,
        email TEXT NOT NULL UNIQUE,
        phone TEXT NOT NULL UNIQUE
    );",
    )
    .unwrap();

    let rx = conn.watch();
    let handle = spawn(move || {
        while let Ok(notification) = rx.recv() {
            println!(
                "Got notification: {:?} {}.{} #{}",
                notification.operation,
                notification.database,
                notification.table,
                notification.row_id
            );
        }
    });

    conn.execute("INSERT INTO contacts (first_name, last_name, email, phone) VALUES (\"Mark\", \"Christian\", \"m@rkchristian.ca\", \"650-421-6262\")").unwrap();
    conn.execute("INSERT INTO contacts (first_name, last_name, email, phone) VALUES (\"Nathalie\", \"Christian\", \"nathalie@mngl.ca\", \"408-204-7759\")").unwrap();
    conn.execute("DELETE FROM contacts where contact_id = 2")
        .unwrap();
    conn.execute("DELETE FROM contacts where contact_id = 1")
        .unwrap();

    println!("Dropping connection");
    drop(conn);

    // sadly, the receiver isn't automatically dropped just because the connection has been dropped,
    // so we will hang here forever:

    println!("Waiting for notification receiver thread to finish");
    handle.join().unwrap();
}
