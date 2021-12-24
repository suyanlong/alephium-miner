use crate::model;
use crate::model::Job;
use std::default::Default;
use std::time;

#[derive(Debug, Clone)]
pub struct Task {
    task_id: u64,
    worker_id: String,
    job: model::Job,           //当前计算的任务
    hash_count: u64,           //当前计算次数
    hash_rate: u64,            //当前任务的算力
    start_time: time::Instant, //单次任务开始计算时间
    end_time: time::Instant,   //单次任务结束计算时间
    nonce: [u8; 24],           //nonce,最终状态的nonce值
    status: usize,             //0:成功，1：limit timeout, 2:被动放弃, 3: 失败
}

impl Default for Task {
    fn default() -> Self {
        Task {
            task_id: 0,
            worker_id: "".to_string(),
            job: Default::default(),
            hash_count: 0,
            hash_rate: 0,
            start_time: time::Instant::now(),
            end_time: time::Instant::now(),
            // ..Default::default()
            nonce: Default::default(),
            status: 0,
        }
    }
}

impl Task {
    pub fn new() -> Task {
        let mut t = Task::default();
        t.start_time = time::Instant::now();
        t.task_id = rand::random();
        t
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    pub fn job(&mut self) -> &mut Job {
        &mut self.job
    }

    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }

    pub fn get_job(self) -> Job {
        self.job
    }

    pub fn status(&self) -> usize {
        self.status
    }

    pub fn with_job(mut self, t: Job) -> Self {
        self.job = t;
        self
    }

    pub fn with_worker_id(mut self, t: String) -> Self {
        self.worker_id = t;
        self
    }

    fn with_start_time(mut self, t: time::Instant) -> Self {
        self.start_time = t;
        self
    }

    fn with_end_time(mut self, t: time::Instant) -> Self {
        self.end_time = t;
        self
    }

    pub fn with_nonce(mut self, t: [u8; 24]) -> Self {
        self.nonce = t;
        self
    }

    pub fn with_hash_count(mut self, t: u64) -> Self {
        self.hash_count = t;
        self
    }

    pub fn hash_count(&self) -> u64 {
        self.hash_count
    }

    pub fn with_status(mut self, t: usize) -> Self {
        self.status = t;
        self
    }

    pub fn build(mut self) -> Self {
        self.end_time = time::Instant::now();
        let consume_time = (self.end_time - self.start_time).as_secs();
        self.hash_rate = self.hash_count / consume_time;
        self
    }
}

#[cfg(test)]
mod tests {
    use chrono;
    use chrono::Timelike;
    #[test]
    fn test_task() {}
}
