use std::{fmt, thread};
use std::sync::{Arc, mpsc, Mutex};
use std::thread::current;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error creating pool")
    }
}

#[derive(Debug)]
pub struct Timing {
    pub parse_http_request: Duration,
    pub parse_request_line: Duration,
    pub parse_status_code: Duration,
    pub create_content: Duration,
    pub create_response: Duration,
    pub stream: Duration,

    current_start: Option<Instant>,
}

impl Timing {
    pub fn new() -> Timing {
        Timing {
            parse_http_request: Duration::from_secs(0),
            parse_request_line: Duration::from_secs(0),
            parse_status_code: Duration::from_secs(0),
            create_content: Duration::from_secs(0),
            create_response: Duration::from_secs(0),
            stream: Duration::from_secs(0),
            current_start: None,
        }
    }

    pub fn start(&mut self) {
        self.current_start = Some(Instant::now());
    }

    pub fn end(&mut self, field: u8) {
        if !self.current_start.is_some() {
            return;
        }

        match self.current_start.take() {
            Some(start) if field == 0 => self.parse_http_request += start.elapsed(),
            Some(start) if field == 1 => self.parse_request_line += start.elapsed(),
            Some(start) if field == 2 => self.parse_status_code += start.elapsed(),
            Some(start) if field == 3 => self.create_content += start.elapsed(),
            Some(start) if field == 4 => self.create_response += start.elapsed(),
            Some(start) => self.stream += start.elapsed(),
            None => {}
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::SyncSender<Job>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    timing: Arc<Mutex<Timing>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let timing = Timing::new();
        let timing_arc = Arc::new(Mutex::new(timing));
        let timing_arc_clone = Arc::clone(&timing_arc);

        let thread = thread::Builder::new()
            .name(format!("Worker {}", id))
            .spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                job(&mut *timing_arc_clone.lock().unwrap());
            }).unwrap();

        Worker { id, thread, timing: Arc::clone(&timing_arc) }
    }
}

type Job = Box<dyn FnOnce(&mut Timing) + Send + 'static>;

impl ThreadPool {
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size < 1 {
            return Err(PoolCreationError);
        }

        let (sender, receiver) = mpsc::sync_channel(1);

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce(&mut Timing) + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }

    pub fn print_stats(&self) {
        println!("Thread pool stats:");
        let mut timing = Timing::new();
        println!("Thread\tParse HTTP\tParse Req\tParse Stat\tCreate Cont\tCreate Resp\tStream");

        for worker in &self.workers {
            // add current timing to total timing
            let mut worker_timing = worker.timing.lock().unwrap();
            timing.parse_http_request += worker_timing.parse_http_request;
            timing.parse_request_line += worker_timing.parse_request_line;
            timing.parse_status_code += worker_timing.parse_status_code;
            timing.create_content += worker_timing.create_content;
            timing.create_response += worker_timing.create_response;
            timing.stream += worker_timing.stream;
            // print current worker stats
            println!("{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                     worker.id,
                     worker_timing.parse_http_request,
                     worker_timing.parse_request_line,
                     worker_timing.parse_status_code,
                     worker_timing.create_content,
                     worker_timing.create_response,
                     worker_timing.stream);
        }

        println!("Total stats: {:?}", timing);
        // print stats with percentages of total time
        let total_time = timing.parse_http_request +
            timing.parse_request_line +
            timing.parse_status_code +
            timing.create_content +
            timing.create_response +
            timing.stream;
        println!("Total time: {:?}", total_time);
        println!("Percentage of total time:");
        println!("Parse HTTP: {}%", timing.parse_http_request.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("Parse Req: {}%", timing.parse_request_line.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("Parse Stat: {}%", timing.parse_status_code.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("Create Cont: {}%", timing.create_content.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("Create Resp: {}%", timing.create_response.as_secs_f64() / total_time.as_secs_f64() * 100.0);
        println!("Stream: {}%", timing.stream.as_secs_f64() / total_time.as_secs_f64() * 100.0);
    }
}
