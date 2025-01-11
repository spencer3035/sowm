use std::path::PathBuf;

fn main() {
    let fifo_path = get_pipe_path();
    println!("Fifo path is {}", fifo_path.display());
}

fn get_pipe_path() -> PathBuf {
    let uid = users::get_current_uid();
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(uid.to_string());
    debug_assert!(p.exists(), "run path doesn't exist: {}", p.display());
    p.push("sowm.fifo");
    p
}
