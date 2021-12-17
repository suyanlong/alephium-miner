use crate::constant;
use crate::model;
use blake3;
use blake3::{hash, Hash};
use std::sync::{atomic, Arc};

pub struct Worker {
    worker_id: String,                //矿工号
    hash_rate: u64,                   //算力
    hash_count: u64,                  //计算次数
    task_count: u64,                  //任务计数
    tasked_count: u64,                //已经完成的任务
    free_tasked_count: u64,           //释放掉的任务
    miner_setup_time: u64,            //每次计算任务的开始时间。
    miner_hash_limit: u64,            //单次任务挖矿最大限制，主动放弃当前任务。
    is_free: Arc<atomic::AtomicBool>, //被动通知需要下拉最新的任务。
    current_task: model::Job,         //当前计算的任务
    nonce: [u8; 24],                  //nonce
}

pub struct ArcRef {
    worker_id: String,
    is_free: Arc<atomic::AtomicBool>,
}

impl Worker {
    fn start() {}

    fn work() {}
    fn reset(&mut self) {
        self.reset_nonce()
    }

    fn arc_ref(&self) -> ArcRef {
        ArcRef {
            worker_id: self.worker_id.clone(),
            is_free: self.is_free.clone(),
        }
    }

    fn reset_nonce(&mut self) {
        self.nonce = self.nonce.each_mut().map(|mut val| {
            *val = rand::random::<u8>();
            *val
        });
    }

    fn mining(&mut self) {
        use std::sync::atomic;
        loop {
            if self.is_free.load(atomic::Ordering::Relaxed) {
                return;
            }
        }
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
