use std::{pin::Pin, sync::Arc};

use crate::args::Args;
use anyhow::Error;
use drain_flow::SimpleDrain;
use parking_lot::{Mutex, RwLock};
use tokio::{
    fs::File,
    io::{stdin, AsyncBufRead, AsyncBufReadExt, BufReader},
    runtime::Builder,
};
use tracing::{debug, instrument};

#[derive(Clone, Debug)]
pub(crate) struct LyreTail {
    drain: Arc<RwLock<SimpleDrain>>,
    args: Arc<Mutex<Args>>,
}

impl LyreTail {
    #[instrument]
    pub(crate) fn create_app(
        drain: Option<Arc<RwLock<SimpleDrain>>>,
        args: Arc<Mutex<Args>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            drain: drain
                .or_else(|| {
                    Some(Arc::new(RwLock::new(
                        SimpleDrain::new(vec![]).expect("creating new drain should work"),
                    )))
                })
                .unwrap(),
            args: args.clone(),
        })
    }

    pub(crate) fn get_drain_ref(&self) -> Arc<RwLock<SimpleDrain>> {
        debug!("cloning drain");
        self.drain.clone()
    }

    // Run encapsulates the main input loop which processes lines and updates the drain
    //
    #[instrument]
    pub fn run(&self) {
        let file = self.args.lock().file.clone();
        let drain = self.get_drain_ref();
        let io_runtime = Builder::new_current_thread().enable_all().build().unwrap();
        std::thread::spawn(move || {
            let mut reader = Box::pin(BufReader::new(stdin())) as Pin<Box<dyn AsyncBufRead>>;
            if let Some(file_path) = file {
                reader = io_runtime.block_on(async {
                    Box::pin(BufReader::new(File::open(file_path).await.unwrap()))
                });
            }
            io_runtime.block_on(async {
                let mut buffer = String::new();
                while let Ok(b) = reader.read_line(&mut buffer).await {
                    if b == 0 {
                        break;
                    }
                    let _ = drain.write().process_line(buffer.clone()).unwrap();
                    buffer.clear();
                }
            });
        });
    }
}
