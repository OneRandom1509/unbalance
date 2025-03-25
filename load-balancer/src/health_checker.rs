use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

pub fn check_health(workers: Vec<String>) {
    thread::spawn(move || {
        let worker_count: usize = workers.len();
        let mut worker_index: usize = 0;
        loop {
            let worker: &String = &workers[worker_index];
            let worker_addr = worker;
            let health = TcpStream::connect(worker_addr).is_ok();

            if !health {
                println!("Worker node {} is down", worker);
            } else {
                println!("Worker node {} is up!", worker);
            }
            worker_index = (worker_index + 1) % worker_count;
            std::thread::sleep(time::Duration::from_secs(10));
        }
    });
}
