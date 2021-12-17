use crate::counter::Counter;
use crate::worker::{ArcRef, Worker};
use std::collections::HashMap;
use std::panic::resume_unwind;
use threadpool;

use crate::model::{Job, WorkUnit};
use crate::task::Task;
use crate::{config, connection, model, Frame, Message};
use std::sync::mpsc as stdmpsc;
use std::sync::{atomic, Arc};
use tokio::sync::mpsc;

//整个矿工的算力（整个机器的算力）
pub struct Miner {
    pool: threadpool::ThreadPool,
    worker_manger: HashMap<String, ArcRef>,
    counter: Counter,
    conf: config::Config,
}

impl Miner {
    pub fn new(conf: config::Config) -> Miner {
        let option = bincode::config::Configuration::standard()
            .with_big_endian()
            .with_no_limit()
            .with_fixed_int_encoding();

        let mut m = Miner {
            pool: threadpool::Builder::default()
                .num_threads(conf.worker_num)
                .thread_name(format!("{}", "miner"))
                .build(),
            worker_manger: HashMap::default(),
            counter: Counter::new(),
            conf,
        };
        m
    }

    pub async fn work(&mut self) {
        let option = bincode::config::Configuration::standard()
            .with_big_endian()
            .with_no_limit()
            .with_fixed_int_encoding();
        let address = format!("{}:{}", self.conf.ip, self.conf.port);
        let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
        let (tx, mut rx) = mpsc::channel::<Task>(100 * self.conf.worker_num);
        for _ in [0..self.conf.worker_num] {
            let mut worker = Worker::new(tx.clone());
            let work_ref = worker.arc_ref();
            self.pool.execute(move || worker.work());
            self.worker_manger.insert(work_ref.work_id(), work_ref);
        }
        let (mut r, mut w) = connection::pair(client);
        let left_half = tokio::spawn(async move {
            loop {
                match r.read_frame().await {
                    Ok(val) => match val {
                        Some(val) => {
                            if let Frame::Bulk(bytes) = val {
                                let (msg, size) = bincode::decode_from_slice::<Message, _>(
                                    bytes.as_ref(),
                                    option,
                                )
                                .expect("decode_from_slice msg error");
                                //send worker
                            }
                        }
                        None => unreachable!(),
                    },
                    Err(err) => error!("read_frame error {}", err),
                }
            }
        });
        let right_half = tokio::spawn(async move {
            loop {
                if let Some(val) = rx.recv().await {
                    let msg = Message::job(val.get_job());
                    let data =
                        bincode::encode_to_vec(msg, option).expect("encode_to_vec msg error");
                    //send server
                    if let Err(err) = w.write_frame(&Frame::Bulk(data.into())).await {
                        error!("write_frame error {}", err);
                    }
                }
            }
        });
        left_half.await;
        right_half.await;
    }
}
