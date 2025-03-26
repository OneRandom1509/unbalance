use std::net::TcpStream;
use std::thread;
use std::time;

use tracing::{info, warn};

pub fn check_health(workers: Vec<String>) {
    thread::spawn(move || {
        let worker_count: usize = workers.len();
        let mut worker_index: usize = 0;
        loop {
            let worker: &String = &workers[worker_index];
            let worker_addr = worker;
            let health = TcpStream::connect(worker_addr).is_ok();

            if !health {
                warn!(name: "[WORKER DOWN]", "Worker node {} is down", worker);
            } else {
                info!(name: "[WORKER UP]", "Worker node {} is up!", worker);
            }
            worker_index = (worker_index + 1) % worker_count;
            std::thread::sleep(time::Duration::from_secs(10));
        }
    });
}
