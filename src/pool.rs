use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "el pool debe tener al menos un hilo");

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .expect("la cola ya está cerrada")
            .send(Box::new(f))
            .expect("los workers ya se apagaron");
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                if thread.join().is_err() {
                    eprintln!("El worker {} terminó con pánico", worker.id);
                }
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::Builder::new()
            .name(format!("worker-{id}"))
            .spawn(move || loop {
                let mensaje = receiver.lock().unwrap().recv();

                match mensaje {
                    Ok(job) => {
                        println!("[worker-{id}] ejecutando trabajo");
                        job();
                    }
                    Err(_) => {
                        println!("[worker-{id}] cola cerrada, apagándome");
                        break;
                    }
                }
            })
            .expect("no se pudo crear el hilo worker");

        Self {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn ejecuta_todos_los_trabajos_en_paralelo() {
        const N: usize = 8;
        let contador = Arc::new(AtomicUsize::new(0));
        let pool = ThreadPool::new(4);

        for _ in 0..N {
            let contador = Arc::clone(&contador);
            pool.execute(move || {
                thread::sleep(Duration::from_millis(10));
                contador.fetch_add(1, Ordering::SeqCst);
            });
        }

        drop(pool);
        assert_eq!(contador.load(Ordering::SeqCst), N);
    }

    #[test]
    #[should_panic]
    fn tamano_cero_no_es_valido() {
        let _ = ThreadPool::new(0);
    }
}
