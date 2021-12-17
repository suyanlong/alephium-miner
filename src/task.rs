use crate::constant;
use crate::model;
use crate::model::Job;

use chrono;
use chrono::Timelike;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Task {
    job: model::Job, //当前计算的任务
    hash_count: u64, //当前计算次数
    hash_rate: u64,  //当前任务的算力
    start_time: u32, //单次任务开始计算时间
    end_time: u32,   //单次任务结束计算时间
    nonce: [u8; 24], //nonce,最终状态的nonce值
    status: usize,   //0:成功，1：limit timeout,2:被动放弃
}

impl Task {
    pub fn new() -> Task {
        let mut t = Task::default();
        t.start_time = chrono::Local::now().second();
        t
    }

    pub fn job(&self) -> &Job {
        &self.job
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

    fn with_start_time(mut self, t: u32) -> Self {
        self.start_time = t;
        self
    }

    fn with_end_time(mut self, t: u32) -> Self {
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

    pub fn with_status(mut self, t: usize) -> Self {
        self.status = t;
        self
    }

    pub fn build(mut self) -> Self {
        self.end_time = chrono::Local::now().second();
        let consume_time = (self.start_time - self.end_time) as u64;
        self.hash_rate = self.hash_count / consume_time;
        self
    }
}
