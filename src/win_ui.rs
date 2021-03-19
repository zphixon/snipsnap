use crate::error::MyError;

use bindings::windows::win32::shell::Shell_NotifyIconA;

pub fn error(msg: &str, e1: MyError, e2: Option<MyError>) {
    msgbox::create(
        "SnipSnap",
        &format!("{}\n\n{:?}, {:?}", msg, e1, e2),
        msgbox::IconType::Error,
    )
    .unwrap();
}
