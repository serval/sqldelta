use std::ffi::{c_char, c_int, c_void, CStr};

use sqlite3_sys::{sqlite3_update_hook, SQLITE_DELETE, SQLITE_INSERT, SQLITE_UPDATE};

fn main() {
    // From https://www.sqlite.org/c3ref/update_hook.html:
    // The first argument to the callback is a copy of the third argument to sqlite3_update_hook(). The
    // second callback argument is one of SQLITE_INSERT, SQLITE_DELETE, or SQLITE_UPDATE, depending on
    // the operation that caused the callback to be invoked. The third and fourth arguments to the
    // callback contain pointers to the database and table name containing the affected row. The final
    // callback parameter is the rowid of the row. In the case of an update, this is the rowid after the
    // update takes place.
    extern "C" fn did_update(
        _context: *mut c_void,
        op: c_int,
        db_name_ptr: *const c_char,
        table_name_ptr: *const c_char,
        row_id: i64,
    ) {
        let op_str = match op {
            SQLITE_DELETE => "delete",
            SQLITE_INSERT => "insert",
            SQLITE_UPDATE => "update",
            _ => return,
        };
        let db_name = unsafe { CStr::from_ptr(db_name_ptr).to_string_lossy() };
        let table_name = unsafe { CStr::from_ptr(table_name_ptr).to_string_lossy() };

        println!("sqlite3_update_hook callback: {op_str} {db_name}.{table_name} #{row_id}");
    }

    let conn = sqlite::open(":memory:").unwrap();
    let raw_ptr = conn.as_raw();
    let mut context = String::from("");
    unsafe {
        sqlite3_update_hook(
            raw_ptr,
            Some(did_update),
            context.as_mut_ptr() as *mut c_void,
        );
    }

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

    conn.execute("INSERT INTO contacts (first_name, last_name, email, phone) VALUES (\"Mark\", \"Christian\", \"m@rkchristian.ca\", \"650-421-6262\")").unwrap();
    conn.execute("INSERT INTO contacts (first_name, last_name, email, phone) VALUES (\"Nathalie\", \"Christian\", \"nathalie@mngl.ca\", \"408-204-7759\")").unwrap();
    conn.execute("DELETE FROM contacts where contact_id = 2")
        .unwrap();
    conn.execute("DELETE FROM contacts where contact_id = 1")
        .unwrap();
}
