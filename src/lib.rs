use sqlite::Connection;
use sqlite3_sys::{
    sqlite3, sqlite3_int64, sqlite3_update_hook, SQLITE_DELETE, SQLITE_INSERT, SQLITE_UPDATE,
};
use std::{
    ffi::{c_char, c_int, c_void, CStr},
    sync::mpsc::{Receiver, Sender},
};

#[derive(Debug)]
pub enum UpdateOperation {
    Delete,
    Insert,
    Update,
}

#[derive(Debug)]
pub struct UpdateNotification {
    pub operation: UpdateOperation,
    pub database: String,
    pub table: String,
    pub row_id: sqlite3_int64,
}

pub trait UpdateNotifications {
    fn drop(&mut self);
    fn watch(&self) -> Receiver<UpdateNotification>;
}

impl UpdateNotifications for Connection {
    fn watch(&self) -> Receiver<UpdateNotification> {
        // This is just a little struct to allow us to pass some data into our callback function
        struct Context {
            sqlite_ptr: *mut sqlite3,
            tx: Sender<UpdateNotification>,
        }

        // From https://www.sqlite.org/c3ref/update_hook.html:
        // The first argument to the callback is a copy of the third argument to sqlite3_update_hook(). The
        // second callback argument is one of SQLITE_INSERT, SQLITE_DELETE, or SQLITE_UPDATE, depending on
        // the operation that caused the callback to be invoked. The third and fourth arguments to the
        // callback contain pointers to the database and table name containing the affected row. The final
        // callback parameter is the rowid of the row. In the case of an update, this is the rowid after the
        // update takes place.
        extern "C" fn did_update(
            context_ptr: *mut c_void,
            op: c_int,
            db_name_ptr: *const c_char,
            table_name_ptr: *const c_char,
            row_id: sqlite3_int64,
        ) {
            let operation = match op {
                SQLITE_DELETE => UpdateOperation::Delete,
                SQLITE_INSERT => UpdateOperation::Insert,
                SQLITE_UPDATE => UpdateOperation::Update,
                _ => return, // invalid operation; ignore
            };
            let db_name = unsafe { CStr::from_ptr(db_name_ptr).to_string_lossy() };
            let table_name = unsafe { CStr::from_ptr(table_name_ptr).to_string_lossy() };

            let context: &Context = unsafe { std::mem::transmute(context_ptr) };
            let notification = UpdateNotification {
                operation,
                database: db_name.to_string(),
                table: table_name.to_string(),
                row_id,
            };
            if context.tx.send(notification).is_err() {
                // channel is closed; remove our hook
                unsafe {
                    sqlite3_update_hook(context.sqlite_ptr, None, std::ptr::null_mut());
                }
            }
        }

        let (tx, rx) = std::sync::mpsc::channel::<UpdateNotification>();

        unsafe {
            let sqlite_ptr = self.as_raw();
            let tx_boxed = Box::new(Context { tx, sqlite_ptr });
            sqlite3_update_hook(
                sqlite_ptr,
                Some(did_update),
                &*tx_boxed as *const Context as *mut c_void,
            );
            Box::leak(tx_boxed);
        }

        rx
    }

    fn drop(&mut self) {
        todo!()
    }
}
