//! Pool de hilos fijo: `N` workers consumiendo trabajos de una cola `mpsc`.
//!
//! Es el mismo patrón del capítulo 20 del libro de Rust, pero aplicado a
//! conexiones HTTPS en vez de a peticiones HTTP sin cifrar.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Un trabajo es cualquier closure que se pueda mover a otro hilo.
/// `FnOnce` porque se ejecuta una sola vez; `Send + 'static` porque cruza
/// un canal entre hilos y no puede tomar prestados datos locales.
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    /// `Option` para poder soltar el emisor en `Drop` y cerrar el canal.
    sender: Option<Sender<Job>>,
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool {
    /// Crea `size` hilos. `size` debe ser > 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "el pool debe tener al menos un hilo");

        let (sender, receiver) = mpsc::channel::<Job>();
        // La cola la comparten todos los workers: Arc para compartir,
        // Mutex para que solo un worker saque cada trabajo.
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Envía un trabajo a la cola. Los workers lo ejecutarán en cuanto puedan.
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

    /// Número de hilos del pool (se usa en /info).
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Al soltar el último emisor, `recv()` devuelve Err en todos los workers
        // y salen de su bucle. Después hacemos join para esperarlos.
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
                // IMPORTANTE: el guard del Mutex se suelta al terminar esta
                // instrucción `let`, ANTES de ejecutar el trabajo. Si lo
                // mantuviéramos durante `job()`, solo un hilo trabajaría a la vez.
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

        // Esperamos a que el pool se suelte al final del ámbito y haga join.
        drop(pool);
        assert_eq!(contador.load(Ordering::SeqCst), N);
    }

    #[test]
    #[should_panic]
    fn tamano_cero_no_es_valido() {
        let _ = ThreadPool::new(0);
    }
}
