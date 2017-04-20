extern crate libc;
extern crate syslog;

use std::convert::AsRef;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::process::exit;

use syslog::{Facility, Severity};


const IDENT: &'static str = "nologin";


// Writes a string to syslog, or to standard output if anything fails.
fn write_log<S>(log: S)
    where S: AsRef<str>
{
    let msg = format!("{}: {}", IDENT, log.as_ref());

    let success = if let Ok(writer) = syslog::unix(Facility::LOG_AUTH) {
        writer.send_3164(Severity::LOG_CRIT, &*msg).is_ok()
    } else {
        false
    };

    if !success {
        println!("{}", msg);
    }
}


// Helpful wrapper that will convert a `*mut c_char` into a `String`, with an optional fallback if
// the pointer is null.
fn convert_cstr(s: *const libc::c_char, fallback: &str) -> String {
    if s.is_null() {
        return fallback.to_string()
    }

    let cstr = unsafe { CStr::from_ptr(s) };
    cstr.to_string_lossy().into_owned()
}


fn get_username() -> String {
    let ptr = unsafe { libc::getlogin() as *const libc::c_char };
    convert_cstr(ptr, "UNKNOWN")
}


fn get_ttyname(fd: libc::c_int) -> String {
    let ptr = unsafe { libc::ttyname(fd) as *const libc::c_char };
    convert_cstr(ptr, "UNKNOWN")
}


fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}


fn main() {
    let username = get_username();
    let tty = get_ttyname(0);

    write_log(format!("Attempted login by {} on {}", username, tty));

    let message = match read_file("/etc/nologin.txt") {
        Ok(s) => s,
        Err(_) => "This account is currently not available.".to_string(),
    };

    println!("{}", message);
    exit(1);
}
