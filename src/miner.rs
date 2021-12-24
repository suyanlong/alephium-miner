use crate::counter::Counter;
use crate::model::Body;
use crate::model::WorkUnit;
use crate::task::Task;
use crate::worker::{Notifier, Worker};
use crate::{config, connection, Frame, Message};
use crossbeam;
use std::clone::Clone;
use std::sync::Arc;
use threadpool;
use tokio::sync::mpsc;

//整个矿工的算力（整个机器的算力）
pub struct Miner {
    pool: threadpool::ThreadPool,
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
        let client = tokio::net::TcpStream::connect(address).await.unwrap();
        let (tcp_tx, mut tcp_rx) = mpsc::channel::<Task>(100 * self.conf.worker_num);
        let (scheduler_tx, scheduler_rx) = mpsc::channel::<Unit>(100 * self.conf.worker_num);
        let scheduler_tx_clone = scheduler_tx.clone();
        let (mut r, mut w) = connection::pair(client);

        let left_half = tokio::spawn(async move {
            loop {
                // info!("------------read_frame-------------");
                match r.read_frame().await {
                    Ok(val) => match val {
                        Some(val) => {
                            // info!("------------read_frame----111---------");
                            if let Frame::Bulk(bytes) = val {
                                // println!("----------{:?}", bytes.len());
                                let (msg, size) = bincode::decode_from_slice::<Message, _>(
                                    bytes.as_ref(),
                                    option,
                                )
                                .expect("decode_from_slice msg error");
                                //send Scheduler
                                // info!("-====--------------size--{:?}", size);
                                scheduler_tx_clone.send(Unit::MSG(msg)).await;
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
                if let Some(val) = tcp_rx.recv().await {
                    scheduler_tx.send(Unit::TASK(val.clone())).await; //send Scheduler
                    if val.status() == 0 {
                        let msg = Message::submit_req(val.into());
                        let data =
                            bincode::encode_to_vec(msg, option).expect("encode_to_vec msg error");
                        //send server
                        if let Err(err) = w.write_frame(&Frame::Bulk(data)).await {
                            error!("write_frame error {}", err);
                        }
                    }
                }
            }
        });

        let mut notifiters = vec![];
        let (tx, rx) = crossbeam::channel::bounded::<WorkUnit>(100000);
        let mut thread_count = 0;
        while thread_count < self.conf.worker_num {
            thread_count += 1;
            let mut worker = Worker::new(tcp_tx.clone(), rx.clone());
            let notifier = worker.notifier();
            self.pool.execute(move || worker.work());
            notifiters.push(Arc::new(notifier));
        }
        let mut scheduler = Scheduler::new()
            .with_rx(scheduler_rx)
            .with_notifier(notifiters)
            .with_sender(tx);

        let scheduler = tokio::spawn(async move { scheduler.work().await });
        scheduler.await;
        left_half.await;
        right_half.await;
    }
}

enum Unit {
    MSG(Message),
    TASK(Task),
}

#[derive(Default)]
pub struct Scheduler {
    rx: Option<mpsc::Receiver<Unit>>,
    sender: Option<crossbeam::channel::Sender<WorkUnit>>,
    notifier: Vec<Arc<Notifier>>,
    pending_tasks: Vec<(Task, bool)>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Default::default()
    }

    pub fn with_rx(mut self, rx: mpsc::Receiver<Unit>) -> Self {
        self.rx = Some(rx);
        self
    }

    pub fn with_sender(mut self, sender: crossbeam::channel::Sender<WorkUnit>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_notifier(mut self, w: Vec<Arc<Notifier>>) -> Self {
        self.notifier = w;
        self
    }

    pub async fn work(&mut self) {
        let mut counter = Counter::new();
        let count = 0;

        loop {
            if let Some(val) = self.rx.as_mut().unwrap().recv().await {
                match val {
                    Unit::MSG(msg) => {
                        match msg.into() {
                            Body::Jobs(jobs) => {
                                //dispatch job
                                for job in jobs {
                                    let task = Task::new().with_job(job);
                                    self.sender
                                        .as_ref()
                                        .unwrap()
                                        .send(WorkUnit::TaskReq(task))
                                        .unwrap();
                                }
                            }
                            Body::SubmitResult(ret) => {
                                let key = format!("{}-{}", ret.from, ret.to);
                                info!("SubmitResult info: {}", key);
                            }
                            _ => unreachable!(),
                        }
                    }
                    Unit::TASK(task) => {
                        self.pending_tasks.push((task.clone(), false));
                        counter.add(task);
                        counter.interval_print();
                    }
                }
            }
        }
    }
}
