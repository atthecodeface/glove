//a Imports
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

//a WorkItem
//tp WorkItem
type WorkItem = Box<dyn FnOnce() + Send + 'static>;

//a ThreadPool
//tp ThreadPool
#[derive(Debug, Default)]
struct ThreadStats {
    delivered: usize,
    completed: usize,
}
impl ThreadStats {
    #[inline]
    fn inc_delivered(&mut self) {
        self.delivered += 1;
    }
    #[inline]
    fn inc_completed(&mut self) {
        self.completed += 1;
    }
    #[inline]
    fn outstanding(&self) -> usize {
        self.delivered - self.completed
    }
}
pub struct ThreadPool {
    workers: Vec<WorkThread>,
    tx: Option<mpsc::Sender<WorkItem>>,
    rx: Option<Arc<Mutex<mpsc::Receiver<WorkItem>>>>,
    stats: Vec<Arc<Mutex<ThreadStats>>>,
}

//ip std::default::Default for ThreadPool
impl std::default::Default for ThreadPool {
    fn default() -> ThreadPool {
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let workers = vec![];
        let stats = vec![];

        ThreadPool {
            workers,
            tx: Some(tx),
            rx: Some(rx),
            stats,
        }
    }
}

//ip ThreadPool
impl ThreadPool {
    /// Create a new [ThreadPool]
    ///
    /// Add in a number of workers
    pub fn new(size: usize) -> ThreadPool {
        let mut pool = ThreadPool::default();
        for _ in 0..size {
            pool.add_thread();
        }
        pool
    }

    //fp add_thread
    /// Add a new thread to the [ThreadPool]
    pub fn add_thread(&mut self) {
        assert!(
            self.rx.is_some(),
            "Must have set up the MPSC channel and not torn it down"
        );
        let thread_id = self.stats.len();
        let stats = Arc::new(Mutex::new(ThreadStats::default()));
        self.stats.push(stats.clone());
        let rx = self.rx.as_ref().unwrap().clone();
        self.workers.push(WorkThread::new(thread_id, rx, stats));
    }

    //mp issue_work
    /// Issue a work item (a callback) to a thread in the [ThreadPool]
    pub fn issue_work<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.as_ref().unwrap().send(Box::new(f)).unwrap();
    }

    //zz All done
}

//ip Drop for ThreadPool
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // drop the MPSC transmitter
        //
        // This causes any thread that does a *later* 'recv' on the
        // channel to get an error, which causes the thread to break
        // out of its work loop and terminate; hence the threads will
        // eventually all be ready to join
        drop(self.tx.take());

        // Join the work threads one at a time
        for wt in &mut self.workers {
            if let Some(thread) = wt.take_thread() {
                eprintln!(
                    "Waiting for work thread {} to have completed",
                    wt.thread_id()
                );
                thread.join().unwrap();
            } else {
                eprintln!(
                    "Bug - work thread {} has already lost its JoinHandle - double drop of thread?",
                    wt.thread_id()
                );
            }
        }
    }
}

//a WorkThread
//tp WorkThread
pub struct WorkThread {
    thread_id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

//ip WorkThread
impl WorkThread {
    //dp take_thread
    /// Take the thread's 'JoinHandle' away and replace it with None;
    /// this must be invoked *once* for each thread when the pool
    /// thinks that the thread is completing; the pool can then 'join'
    /// with this handle
    fn take_thread(&mut self) -> Option<thread::JoinHandle<()>> {
        self.thread.take()
    }

    //ap thread_id
    /// Get the UID of the thread
    fn thread_id(&self) -> usize {
        self.thread_id
    }

    //cp new
    /// Create a new thread watching an MPSC rx channel for work
    fn new(
        thread_id: usize,
        rx: Arc<Mutex<mpsc::Receiver<WorkItem>>>,
        stats: Arc<Mutex<ThreadStats>>,
    ) -> WorkThread {
        let thread = thread::spawn(move || loop {
            // Get the next WorkItem - or Err(_) if the pool has killed the MPSC transmitter
            let work_item_or_err = rx.lock().unwrap().recv();

            // Handle the work and loop;
            let Ok(work_item) = work_item_or_err else {
                eprintln!("WorkThread {thread_id} disconnected; shutting down.");
                break;
            };
            stats.lock().map(|mut s| s.inc_delivered());
            work_item();
            stats.lock().map(|mut s| s.inc_completed());
        });

        WorkThread {
            thread_id,
            thread: Some(thread),
        }
    }

    //zz All done
}
