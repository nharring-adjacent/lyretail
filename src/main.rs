use std::io;

use joinery::JoinableIterator;

use drain_flow::SimpleDrain;

fn main() {
    let mut drain = SimpleDrain::new(vec![]).unwrap();
    let mut buffer = String::new();
    while let Ok(b) = io::stdin().read_line(&mut buffer) {
        if b == 0 {
            break;
        }
        let _ = drain.process_line(buffer.clone()).unwrap();
        buffer.clear();
    }
    let groups = drain.iter_groups();
    println!("{}", groups.iter().flatten().map(|g| {g.to_string()}).join_with("\n").to_string());
}
