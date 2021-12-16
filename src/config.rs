#[derive(Debug, Clone, Default)]
pub struct Config {
    pub ip: String,
    pub port: String,
    pub miner_type: String,
    pub worker_num: usize,
}
