use crate::task::Task;
use std::collections::HashMap;
use std::time;

#[derive(Debug, Clone)]
pub struct Counter {
    tasks: HashMap<u64, Task>,
    total_hash_count: u64,           //计算总次数
    succeed_tasked_count: u64,       //已经完成的任务数
    free_tasked_count: u64,          //释放掉的任务数
    miner_start_time: time::Instant, //每次计算任务的开始时间。
    print_setup_time: time::Instant,
    interval: u64,

    // total_task_count: u64,     //总共接收到任务数
    // succeed_tasks: Vec<Task>,  //成功
    // free_tasks: Vec<Task>,     //失败
    // miner_end_time: u32,       //每次计算任务的结束时间。
    // miner_hash_limit: u64,     //任务挖矿最大限制，主动放弃当前任务。
}

impl Default for Counter {
    fn default() -> Self {
        Counter {
            tasks: Default::default(),
            total_hash_count: 0,
            succeed_tasked_count: 0,
            free_tasked_count: 0,
            miner_start_time: time::Instant::now(),
            print_setup_time: time::Instant::now(),
            interval: 0,
        }
    }
}

impl Counter {
    pub fn new() -> Counter {
        let mut c = Counter::default();
        c.interval = 60 * 2;
        c
    }

    pub fn add(&mut self, mut task: Task) {
        self.update_count(&task);
        self.tasks.insert(task.task_id(), task);
    }

    fn update_count(&mut self, task: &Task) {
        self.total_hash_count += task.hash_count();
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
        let end_time = time::Instant::now();
        self.total_hash_count / (end_time - self.miner_start_time).as_secs()
    }

    pub fn task_rate(&self) -> u64 {
        let end_time = time::Instant::now();
        self.tasks.len() as u64 / (end_time - self.miner_start_time).as_secs()
    }

    pub fn interval_print(&mut self) {
        let now = time::Instant::now();
        if (now - self.print_setup_time).as_secs() > self.interval {
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
