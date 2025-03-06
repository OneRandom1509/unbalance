use dotenv::dotenv;
use std::env;
fn round_robin_scheduler(){
    let mut env_variable =dotenv().ok();
    let mut workers = env::var("WORKER_NODES").unwrap_or("".to_string());

    if(!env_variable){
        println!("env variable WORKER_NODES has not been set");
        return;
    }

    let mut worker_list = worker.split(",").collect::<Vec<&str>>();
    let mut worker_count = worker_list.len();
    let mut worker_index =0;
    loop{
        let worker = worker_list[worker_index];
        println!("Request sent to worker node at {}",worker);
        worker_index = (worker_index+1)% worker_count;


    }
}