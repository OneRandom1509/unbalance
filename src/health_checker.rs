use std::net::{TcpStream,SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time;



pub fn check_health(workers:Vec<String>){
        let mut worker_count:usize = workers.len();
        let mut worker_index:usize = 0;
        loop{
                let worker:&String = &workers[worker_index];
                let worker_addr:Result<SocketAddr,_> = worker.parse().unwrap();
                let health =TcpStream::connect(worker_addr).is_ok();
                if !health{
                    println!("Worker node at {} is down",worker);
                }
                    worker_index = (worker_index+1)% worker_count;
                    std::thread::sleep(time::Duration::from_secs(10));
                }
            }
        