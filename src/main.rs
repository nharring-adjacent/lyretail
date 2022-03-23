mod args;

use std::{
    io::{stdout, Write},
    pin::Pin,
    sync::{Arc, RwLock},
};

use clap::Parser;
use crossterm::{
    queue,
    style::Print,
    terminal::{Clear, ClearType},
};
use drain_flow::SimpleDrain;
use joinery::JoinableIterator;
use tokio::{
    fs::File,
    io::{stdin, AsyncBufRead, AsyncBufReadExt, BufReader},
    task, time,
};

use crate::args::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let drain = Arc::new(RwLock::new(SimpleDrain::new(vec![]).unwrap()));
    let exit_drain = drain.clone();
    ctrlc::set_handler(move || println!("{}", print_groups(&exit_drain.read().unwrap()).unwrap()))
        .expect("Installing ctrl-c handler works");
    let mut buffer = String::new();
    if args.periodic {
        let period_drain = drain.clone();
        let period_interval = args.interval.clone();
        task::spawn(async move {
            print_update(period_interval, period_drain).await;
        });
    };
    let mut reader = Box::pin(BufReader::new(stdin())) as Pin<Box<dyn AsyncBufRead>>;
    if let Some(file_path) = args.file {
        reader = Box::pin(BufReader::new(File::open(file_path).await.unwrap()));
    }
    while let Ok(b) = reader.read_line(&mut buffer).await {
        if b == 0 {
            break;
        }
        let _ = drain.write().unwrap().process_line(buffer.clone()).unwrap();
        buffer.clear();
    }
    if args.periodic {
        let mut stdout = stdout();
        queue!(stdout, Clear(ClearType::All)).unwrap();
        stdout.flush().unwrap();
    }
    if let Some(s) = print_groups(&drain.read().unwrap()) {
        println!("{}", s);
    };
}

fn print_groups(drain: &SimpleDrain) -> Option<String> {
    let groups = drain.iter_groups();
    Some(format!(
        "{}",
        groups
            .iter()
            .flatten()
            .map(|g| { g.to_string() })
            .join_with("\n")
            .to_string()
    ))
}

async fn print_update(freq: chrono::Duration, drain: Arc<RwLock<SimpleDrain>>) {
    let mut interval = time::interval(freq.to_std().expect("chrono can convert to std duration"));
    let mut stdout = stdout();
    loop {
        interval.tick().await;
        if let Some(s) = print_groups(&drain.read().unwrap()) {
            queue!(stdout, Clear(ClearType::All), Print(s)).unwrap();
            stdout.flush().unwrap();
        }
    }
}
