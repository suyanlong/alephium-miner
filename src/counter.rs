use crate::task::Task;
use chrono;
use chrono::Timelike;
use std::collections::HashMap;
use std::thread::sleep;

#[derive(Debug, Clone, Default)]
pub struct Counter {
    tasks: HashMap<u64, Task>,
    // succeed_tasks: Vec<Task>,  //成功
    // free_tasks: Vec<Task>,     //失败
    total_hash_count: u64, //计算总次数
    // total_task_count: u64,     //总共接收到任务数
    succeed_tasked_count: u64, //已经完成的任务数
    free_tasked_count: u64,    //释放掉的任务数
    miner_start_time: u32,     //每次计算任务的开始时间。
    print_setup_time: u32,
    interval: u32,
    // miner_end_time: u32,       //每次计算任务的结束时间。
    // miner_hash_limit: u64,     //任务挖矿最大限制，主动放弃当前任务。
}

impl Counter {
    pub fn new() -> Counter {
        let mut c = Counter::default();
        c.print_setup_time = chrono::Local::now().second();
        c.interval = 60 * 2;
        c.miner_start_time = chrono::Local::now().second();
        c
    }

    pub fn add(&mut self, mut task: Task) {
        self.update_count(&task);
        self.tasks.insert(task.task_id(), task);
    }

    fn update_count(&mut self, task: &Task) {
        match task.status() {
            0 => {
                self.succeed_tasked_count += 1;
                // self.succeed_tasks.push(task.clone());
            }
            1 => {
                self.free_tasked_count += 1;
            }
            2 => {
                self.free_tasked_count += 1;
            }
            _ => unreachable!(),
        }
    }

    pub fn update_task_status(&mut self, task_id: u64, status: usize) {
        // self.tasks
        //     .get_mut(&task_id)
        //     .map(|task| (*task).with_status(status));
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
        self.tasks.len() as u64 / ((end_time - self.miner_start_time) as u64)
    }

    pub fn interval_print(&mut self) {
        let now = chrono::Local::now().second();
        if self.print_setup_time + self.interval < now {
            self.print_setup_time = now;
            info!(
                "total hash count: {}, free_tasked_count: {} task count: {}, hash rate: {}, task rate:{}, ",
                self.total_hash_count,
                // self.succeed_tasked_count,
                self.free_tasked_count,
                self.tasks.len(),
                self.hash_rate(),
                self.task_rate()
            );
        }
    }
}
