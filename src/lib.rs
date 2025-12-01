#![feature(nonpoison_mutex)]
#![feature(sync_nonpoison)]

mod mutex;
pub use mutex::Mutex;