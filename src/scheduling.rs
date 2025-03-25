use std::net::TcpStream;
use std::env;
use main::Message;
use std::io::Write;
pub fn round_robin_scheduler(workers:Vec<String>,message:Message){;
    let mut worker_count = workers.len();
    let mut worker_index =0;
    loop{
        let worker:String = workers[worker_index];
        match TcpStream::connect(worker){
        Ok(mut stream)=>{
            let msg  = stream.write_all(message.content.as_bytes()).unwrap();
            stream.flush().unwrap();
            println!("Request sent to worker node at {}",worker);
            worker_index = (worker_index+1)% worker_count;
        }
        Err(e) =>{
            println!("Error connecting to worker node at {}",worker);
            worker_index = (worker_index+1)% worker_count;
        }
    }
       
    }
    }
