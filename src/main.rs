use std::{io::ErrorKind, os::unix::fs::symlink, path::Path};

use chrono::{DateTime, Utc};
use fanotify::{
    high_level::{Fanotify, FanotifyMode},
    low_level::FAN_CLOSE_WRITE,
};

fn build_tree(path: &Path, pid: i32, base: &Path) {
    let mut path = base.join(path.strip_prefix("/").unwrap());
    path.push("");

    if let Err(e) = std::fs::create_dir_all(&path) {
        if e.kind() != ErrorKind::AlreadyExists {
            panic!("failed to build output tree for {path:?}: {e}");
        }
    }

    let exe_path = format!("/proc/{pid}/exe");
    match std::fs::read_link(&exe_path) {
        Ok(target) => {
            path.push(target.file_name().unwrap());

            if let Err(e) = symlink(target, &path) {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("failed to build symlink for {path:?}: {e}");
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                panic!("failed to build symlink for {exe_path:?}: {e}");
            }
        }
    };
}

fn main() {
    let fa = Fanotify::new_blocking(FanotifyMode::NOTIF).unwrap();
    fa.add_mountpoint(FAN_CLOSE_WRITE, "/").unwrap();

    let age = std::fs::metadata("/").unwrap().created().unwrap();
    let age: DateTime<Utc> = age.into();
    let age = age.format("%Y-%m-%d_%H:%M:%S").to_string();

    let new_out_path = Path::new("/persistent/panoptes").join(age);
    let my_pid = std::process::id() as i32;
    println!("Started listening! (my pid: {my_pid})");

    loop {
        let events = fa.read_event();
        for i in events {
            if i.pid == my_pid {
                continue;
            }

            build_tree(Path::new(&i.path), i.pid, &new_out_path);
        }
    }
}
