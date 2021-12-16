use crate::worker::ArcRef;
use std::collections::HashMap;
use threadpool;

pub struct Miner {
    pool: threadpool::ThreadPool,
    worker_manger: HashMap<u32, ArcRef>,
}

impl Miner {
    fn new(num: usize) -> Miner {
        Miner {
            pool: threadpool::Builder::default()
                .num_threads(num)
                .thread_name(format!("{}", "miner"))
                .build(),
            worker_manger: HashMap::default(),
        }
    }
}
