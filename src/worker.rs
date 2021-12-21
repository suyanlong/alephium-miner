use crate::counter::Counter;
use crate::model;
use crate::model::{Job, WorkUnit};
use crate::task::Task;
use crate::{constant, Message};
use blake3;
use blake3::{hash, Hash};
use chrono;
use chrono::Timelike;
use crossbeam::channel;
use crossbeam::queue;
use futures::future::join;
use std::collections::HashMap;
use std::mem::take;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc as stdmpsc;
use std::sync::{atomic, Arc};
use std::thread::sleep;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use uuid;

//单个线程的算力
pub struct Worker {
    worker_id: String,                //矿工号
    counter: Counter,                 //统计器
    miner_hash_limit: u64,            //单次任务挖矿最大限制，主动放弃当前任务。
    current_nonce: [u8; 24],          //当前nonce
    increase_nonce: u64,              //递增值
    is_free: Arc<atomic::AtomicBool>, //被动通知需要下拉最新的任务。true: 被通知，false: 不需要。
    sender: mpsc::Sender<Task>,       //???
    rx: channel::Receiver<model::WorkUnit>,
    // current_task: Task,                   //当前计算的任务
}

#[derive(Default)]
pub struct Notifier {
    id: String,
    is_free: Arc<atomic::AtomicBool>,
}

impl Notifier {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn notify(&mut self) {
        self.is_free.store(true, atomic::Ordering::Relaxed);
    }
}

impl Worker {
    pub fn new(sender: Sender<Task>, rx: channel::Receiver<model::WorkUnit>) -> Worker {
        Worker {
            worker_id: uuid::Uuid::default().to_string(),
            miner_hash_limit: constant::MINING_STEPS,
            is_free: Arc::new(Default::default()),
            // current_task: Default::default(),
            current_nonce: Default::default(),
            counter: Counter::new(),
            increase_nonce: 0,
            sender,
            rx,
        }
    }

    pub fn work(&mut self) {
        match self.rx.recv() {
            Ok(val) => match val {
                WorkUnit::TaskReq(mut task) => {
                    let job = task.job();
                    let (status, count) = self.mining(job);
                    info!("worker id: {}, from: {}, to: {}, target: {:?}, header: {:?}, current_nonce: {:?}, status: {}",
                           self.worker_id,job.from,job.to,job.target,job.header,self.current_nonce,status);
                    let task = task
                        .with_worker_id(self.worker_id.clone())
                        .with_nonce(self.current_nonce)
                        .with_status(status)
                        .with_hash_count(count)
                        .build();

                    self.counter.add(task.clone());
                    self.counter.inc_hash_count(count);
                    self.sender.blocking_send(task);
                }
                WorkUnit::TaskRes(job_id, ret) => {
                    //应答
                }
            },
            Err(err) => {
                error!("worker: {} recv data error: {}", self.worker_id, err);
            }
        }
    }

    fn increase_nonce(&mut self) {
        self.increase_nonce += 1;
    }

    fn reset(&mut self) {
        self.reset_nonce()
    }

    fn mining(&mut self, job: &mut Job) -> (usize, u64) {
        use std::sync::atomic;
        let mut step_count = 0;
        let mut total_count = 0;
        self.is_free.store(false, atomic::Ordering::Relaxed);
        loop {
            let double_hash = self.double2(job);
            let is = Worker::check_hash(
                double_hash,
                job.target.clone(),
                job.from.clone(),
                job.to.clone(),
            );
            self.increase_nonce();
            step_count += 1;
            total_count += 1;
            if is {
                break (0, total_count);
            }
            if step_count > self.miner_hash_limit {
                if !self.rx.is_empty() {
                    break (1, total_count);
                }
                if self.is_free.load(atomic::Ordering::Relaxed) {
                    break (1, total_count);
                }
                step_count = 0;
            }
            if total_count > self.miner_hash_limit * 100 {
                break (2, total_count);
            }
        }
    }

    pub fn notifier(&self) -> Notifier {
        Notifier {
            id: self.worker_id.clone(),
            is_free: self.is_free.clone(),
        }
    }

    fn reset_nonce(&mut self) {
        self.current_nonce = self.current_nonce.each_mut().map(|mut val| {
            *val = rand::random::<u8>();
            *val
        });
    }

    fn double2(&mut self, job: &Job) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.current_nonce);
        hasher.update(job.header.as_slice());
        let hash1 = hasher.finalize();

        let mut hasher = blake3::Hasher::new();
        hasher.update(hash1.as_bytes());
        hasher.finalize().as_bytes().to_vec()
    }

    fn check_target(hash: Vec<u8>, target: Vec<u8>) -> bool {
        let zero_len = 32 - target.len();
        let (zero_hash, non_zero_hash) = hash.split_at(zero_len);
        for zero in zero_hash {
            if *zero != 0u8 {
                return false;
            }
        }
        let mut i = 0;
        for target_bytes in target {
            if non_zero_hash[i] > target_bytes {
                return false;
            } else if non_zero_hash[i] < target_bytes {
                return true;
            }
            i += 1;
        }
        true
    }

    fn check_hash(hash: Vec<u8>, target: Vec<u8>, from: u32, to: u32) -> bool {
        Worker::check_target(hash.clone(), target) && Worker::check_index(hash, from, to)
    }

    fn check_index(hash: Vec<u8>, from: u32, to: u32) -> bool {
        let big_index = (hash[31] % (constant::CHAIN_NUMS as u8)) as u32;
        (big_index / constant::GROUP_NUMS == from) && (big_index % constant::GROUP_NUMS == to)
    }

    fn double(input: &[u8]) -> Hash {
        blake3::hash(blake3::hash(input).as_bytes())
    }

    fn hex_to_string(input: &[u8]) -> String {
        hex::encode(input)
    }

    fn chain_index(from: u32, to: u32) -> u32 {
        from * constant::CHAIN_NUMS + to
    }
}

#[cfg(test)]
mod tests {
    use crate::worker::Worker;

    #[test]
    fn test_double() {
        let double_hash = Worker::double(b"foobarbaz");
        // Hash an input incrementally.
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"foo");
        hasher.update(b"bar");
        hasher.update(b"baz");
        let hash1 = hasher.finalize();
        let hash2 = blake3::hash(hash1.as_bytes());
        assert_eq!(double_hash, hash2);
    }

    #[test]
    fn test_check_index() {
        let hex_str = "00000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaae";
        let hash = hex::decode(hex_str).unwrap();
        assert!(Worker::check_index(hash.clone(), 3, 2));
        assert!(!Worker::check_index(hash, 3, 3));
    }

    #[test]
    fn test_check_target() {
        let hash = hex::decode("00000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
        let target =
            hex::decode("00000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap();
        assert!(Worker::check_target(hash.clone(), target));

        // remove 4 leading zeros
        let target =
            hex::decode("0000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        assert!(Worker::check_target(hash.clone(), target));

        // remove all leading zeros
        let target =
            hex::decode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        assert!(Worker::check_target(hash.clone(), target));

        // remove all leading zeros + "aa"
        let target = hex::decode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        assert!(!Worker::check_target(hash.clone(), target));

        // replace leading "aa" with "bb"
        let target =
            hex::decode("bbaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        assert!(Worker::check_target(hash.clone(), target));

        // replace the last "a" with "b"
        let target =
            hex::decode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab").unwrap();
        assert!(Worker::check_target(hash.clone(), target));

        // replace the last "a" with "9"
        let target =
            hex::decode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa9").unwrap();
        assert!(!Worker::check_target(hash.clone(), target));

        let target =
            hex::decode("a9aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        assert!(!Worker::check_target(hash.clone(), target));
    }
}
