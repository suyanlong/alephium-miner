use crate::task::Task;
use chrono;
use chrono::Timelike;
use std::collections::HashMap;
use std::thread::sleep;

#[derive(Debug, Clone, Default)]
pub struct Counter {
    tasks: HashMap<String, Task>,
    succeed_tasks: Vec<Task>,  //成功
    free_tasks: Vec<Task>,     //失败
    total_hash_count: u64,     //计算总次数
    total_task_count: u64,     //总共接收到任务数
    succeed_tasked_count: u64, //已经完成的任务数
    free_tasked_count: u64,    //释放掉的任务数
    miner_start_time: u32,     //每次计算任务的开始时间。
                               // miner_end_time: u32,       //每次计算任务的结束时间。
                               // miner_hash_limit: u64,     //任务挖矿最大限制，主动放弃当前任务。
}

impl Counter {
    pub fn new() -> Counter {
        let mut c = Counter::default();
        c.miner_start_time = chrono::Local::now().second();
        c
    }

    pub fn add(&mut self, task: Task) {
        let job = task.job().clone();
        let key = format!(
            "{}-{}-{}",
            job.from,
            job.to,
            hex::encode(job.header.clone()) //TODO ?
        );
        self.tasks.insert(key, task.clone());
        self.total_task_count += 1;
        match task.status() {
            0 => {
                self.succeed_tasked_count += 1;
                self.succeed_tasks.push(task.clone());
            }
            1 => {
                self.free_tasked_count += 1;
                self.free_tasks.push(task);
            }
            2 => {
                self.free_tasked_count += 1;
                self.free_tasks.push(task);
            }
            _ => unreachable!(),
        }
    }

    pub fn inc_hash_count(&mut self, count: u64) {
        self.total_hash_count += count;
    }

    pub fn hash_rate(&self) -> u64 {
        let end_time = chrono::Local::now().second();
        self.total_hash_count / ((end_time - self.miner_start_time) as u64)
    }

    pub fn task_rate(&self) -> u64 {
        let end_time = chrono::Local::now().second();
        self.total_task_count / ((end_time - self.miner_start_time) as u64)
    }
}
