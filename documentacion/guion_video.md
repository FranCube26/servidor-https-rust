# Guion del video: explicando `servidor-https-rust` línea por línea

**Duración estimada:** 9-10 minutos  
**Estructura:** primero habla **Francisco Sevilla** (`Cargo.toml`, `src/main.rs` y `src/pool.rs`) y después **Yeimar Gonzalez** (`src/tls.rs` y `src/http.rs`).  
**Idea:** explicar qué hace cada línea no vacia del código, sin profundizar en teoría.

> Convención: **EN PANTALLA** = qué mostrar; **DI** = texto literal a decir.

**Nota:** Las líneas en blanco solo separan bloques y no ejecutan nada. Por eso no tienen viñeta.

## Orden de la explicación

- 0:00 - 0:20: introducción, **Francisco Sevilla**.
- 0:20 - 0:50: `Cargo.toml`, **Francisco Sevilla**.
- 0:50 - 3:20: `src/main.rs`, **Francisco Sevilla**.
- 3:20 - 5:40: `src/pool.rs`, **Francisco Sevilla**.
- 5:40 - 5:50: transición a **Yeimar Gonzalez**.
- 5:50 - 6:50: `src/tls.rs`, **Yeimar Gonzalez**.
- 6:50 - 9:10: `src/http.rs`, **Yeimar Gonzalez**.
- 9:10 - 9:45: demostración, **Yeimar Gonzalez**.
- 9:45 - 10:00: cierre de **Francisco Sevilla** y **Yeimar Gonzalez**.

---

# PARTE 1 — Francisco Sevilla

## 0:00 - 0:20 · Introducción

**EN PANTALLA:** título del proyecto con los nombres Francisco Sevilla y Yeimar Gonzalez.

**DI (Francisco):**
> "Hola. Vamos a ver qué hace cada línea de nuestro servidor HTTPS. Yo explico el archivo de configuración, el programa principal y el pool de hilos; después Yeimar explica TLS y HTTP. Al final lo ejecutamos."

## 0:20 - 3:20 · Archivos de Francisco Sevilla

### `Cargo.toml` (línea por línea)

**EN PANTALLA:** abrir `Cargo.toml` y recorrerlo de arriba abajo.

- Línea 1: `[package]` → abre la sección con los datos del paquete
- Línea 2: `name = "servidor-https-rust"` → pone el nombre del proyecto
- Línea 3: `version = "0.1.0"` → pone la version del proyecto
- Línea 4: `edition = "2021"` → elige la edicion del lenguaje Rust
- Línea 5: `authors = ["Francisco Sevilla", "Yeimar Gonzalez"]` → guarda los nombres de los autores
- Línea 6: `description = "Servidor HTTPS multihilo en Rust con pool de hilos y rustls"` → describe el proyecto en una frase
- Línea 8: `[dependencies]` → abre la lista de librerías que usa el proyecto
- Línea 9: `rustls = { version = "0.23", default-features = false, features = ["std", "ring", "tls12"] }` → añade rustls para hacer HTTPS con ring y TLS 1.2
- Línea 10: `rustls-pemfile = "2"` → añade la libreria que lee archivos PEM
- Línea 11: `rcgen = { version = "0.13", default-features = false, features = ["ring", "pem"] }` → añade rcgen para crear certificados de prueba
- Línea 12: `num_cpus = "1"` → añade la libreria que cuenta los procesadores
- Línea 14: `[profile.release]` → abre la configuración de la compilación optimizada
- Línea 15: `opt-level = 3` → pide la máxima optimizacion al compilar
- Línea 16: `strip = true` → quita simbolos de depuracion y hace el ejecutable mas pequeño

**DI (Francisco):**
> "Aquí están solo las librerías que usamos: rustls para HTTPS, rustls-pemfile para leer certificados, rcgen para crear uno de prueba y num_cpus para elegir el número de hilos. También configuramos la compilación optimizada."

### `src/main.rs` (línea por línea)

**EN PANTALLA:** abrir `src/main.rs` y seguir el orden de las líneas.

- Línea 1: `mod http;` → incluye el archivo src/http.rs en el proyecto
- Línea 2: `mod pool;` → incluye el archivo src/pool.rs en el proyecto
- Línea 3: `mod tls;` → incluye el archivo src/tls.rs en el proyecto
- Línea 5: `use std::env;` → trae la funcion para leer los argumentos del programa
- Línea 6: `use std::net::TcpListener;` → trae TcpListener para escuchar conexiones entrantes
- Línea 7: `use std::path::PathBuf;` → trae PathBuf para guardar rutas de archivos
- Línea 8: `use std::sync::Arc;` → trae Arc para compartir datos entre hilos
- Línea 9: `use std::time::Instant;` → trae Instant para medir el tiempo
- Línea 11: `use rustls::ServerConfig;` → trae la configuración del servidor TLS
- Línea 13: `struct Args {` → abre la estructura con las opciones del programa
- Línea 14: `    addr: String,` → guarda la direccion IP y el puerto
- Línea 15: `    threads: usize,` → guarda cuantos hilos workers crear
- Línea 16: `    cert: Option<PathBuf>,` → guarda la ruta del certificado si se dio
- Línea 17: `    key: Option<PathBuf>,` → guarda la ruta de la clave si se dio
- Línea 18: `}` → cierra la estructura Args
- Línea 20: `impl Args {` → abre los métodos de Args
- Línea 21: `    fn parse() -> Result<Self, String> {` → abre la funcion que lee y valida opciones; devuelve Args o error
- Línea 22: `        let mut addr = "127.0.0.1:8443".to_string();` → pone la direccion por defecto 127.0.0.1:8443
- Línea 23: `        let mut threads = num_cpus::get();` → pone por defecto un hilo por procesador
- Línea 24: `        let mut cert = None;` → empieza sin ruta de certificado
- Línea 25: `        let mut key = None;` → empieza sin ruta de clave
- Línea 27: `        let mut iter = env::args().skip(1);` → crea un iterador de argumentos sin el nombre del programa
- Línea 28: `        while let Some(arg) = iter.next() {` → recorre cada argumento recibido
- Línea 29: `            match arg.as_str() {` → compara el argumento con las opciones conocidas
- Línea 30: `                "-h" | "--help" => {` → si es ayuda, abre este caso
- Línea 31: `                    println!(` → abre la impresion del texto de ayuda
- Línea 32: `                        "Uso: servidor-https-rust [opciones]\n\` → muestra la línea de uso del programa
- Línea 33: `                         \x20   --addr <IP:PUERTO>   Dirección de escucha (por defecto 127.0.0.1:8443)\n\` → explica la opcion --addr en la ayuda
- Línea 34: `                         \x20   --threads <N>       Número de hilos (por defecto: número de CPUs)\n\` → explica la opcion --threads en la ayuda
- Línea 35: `                         \x20   --cert <ruta>       Certificado PEM (si no se da, se genera uno autofirmado)\n\` → explica la opcion --cert en la ayuda
- Línea 36: `                         \x20   --key <ruta>        Clave privada PEM\n\` → explica la opcion --key en la ayuda
- Línea 37: `                         \x20   -h, --help          Mostrar esta ayuda"` → explica la opcion -h y --help en la ayuda
- Línea 38: `                    );` → cierra la impresion del texto de ayuda
- Línea 39: `                    std::process::exit(0);` → termina el programa con exito después de mostrar la ayuda
- Línea 40: `                }` → cierra el caso de ayuda
- Línea 41: `                "--addr" => addr = iter.next().ok_or("--addr necesita un valor")?,` → guarda la direccion que viene después de --addr o falla
- Línea 42: `                "--threads" => {` → abre el caso que lee el número de hilos
- Línea 43: `                    let raw = iter.next().ok_or("--threads necesita un valor")?;` → toma el valor de --threads o devuelve error
- Línea 44: `                    threads = raw.parse().map_err(|_| "--threads debe ser un número")?;` → convierte el texto a número o devuelve error
- Línea 45: `                }` → cierra el caso --threads
- Línea 46: `                "--cert" => {` → abre el caso que lee la ruta del certificado
- Línea 47: `                    cert = Some(PathBuf::from(` → empieza a guardar la ruta del certificado
- Línea 48: `                        iter.next().ok_or("--cert necesita un valor")?,` → toma la ruta o devuelve error
- Línea 49: `                    ))` → cierra el guardado de la ruta
- Línea 50: `                }` → cierra el caso --cert
- Línea 51: `                "--key" => key = Some(PathBuf::from(iter.next().ok_or("--key necesita un valor")?)),` → guarda la ruta de la clave o devuelve error
- Línea 52: `                other => return Err(format!("opción desconocida: {other}")),` → si la opcion no existe, devuelve un error
- Línea 53: `            }` → cierra el match de opciones
- Línea 54: `        }` → cierra el bucle de argumentos
- Línea 56: `        if threads == 0 {` → comprueba que no pidan cero hilos
- Línea 57: `            return Err("--threads debe ser al menos 1".to_string());` → devuelve error si no hay hilos
- Línea 58: `        }` → cierra la comprobacion
- Línea 59: `        if cert.is_some() != key.is_some() {` → comprueba que certificado y clave vayan juntos
- Línea 60: `            return Err("debes pasar --cert y --key juntos, o ninguno de los dos".to_string());` → devuelve error si solo se pasa uno de los dos
- Línea 61: `        }` → cierra la comprobacion
- Línea 63: `        Ok(Self {` → abre la creacion de Args con los valores ya listos
- Línea 64: `            addr,` → mete la direccion en Args
- Línea 65: `            threads,` → mete el número de hilos en Args
- Línea 66: `            cert,` → mete la ruta del certificado en Args
- Línea 67: `            key,` → mete la ruta de la clave en Args
- Línea 68: `        })` → cierra la creacion de Args
- Línea 69: `    }` → cierra la funcion parse
- Línea 70: `}` → cierra el impl de Args
- Línea 72: `fn main() -> Result<(), Box<dyn std::error::Error>> {` → abre la funcion principal; devuelve error si algo falla
- Línea 73: `    rustls::crypto::ring::default_provider()` → selecciona el proveedor de cifrado ring
- Línea 74: `        .install_default()` → lo instala como proveedor por defecto
- Línea 75: `        .map_err(|_| "no se pudo instalar el proveedor crypto de ring")?;` → convierte un fallo en mensaje y sale si no se pudo instalar
- Línea 77: `    let args = Args::parse()?;` → lee las opciones de la línea de comandos
- Línea 79: `    let config: Arc<ServerConfig> = Arc::new(tls::build_server_config(` → abre la creacion de la configuración TLS que se compartira
- Línea 80: `        args.cert.as_deref(),` → pasa la ruta del certificado si existe
- Línea 81: `        args.key.as_deref(),` → pasa la ruta de la clave si existe
- Línea 82: `    )?);` → cierra la creacion y sale si hay error
- Línea 84: `    let pool = pool::ThreadPool::new(args.threads);` → crea el pool con los hilos pedidos
- Línea 85: `    let listener = TcpListener::bind(&args.addr)?;` → abre el puerto y se queda escuchando
- Línea 87: `    println!(` → abre el mensaje de arranque
- Línea 88: `        "Servidor HTTPS escuchando en https://{}",` → pone el texto del mensaje con la direccion
- Línea 89: `        listener.local_addr()?` → obtiene la direccion real del puerto
- Línea 90: `    );` → cierra el mensaje de arranque
- Línea 91: `    println!("Hilos workers: {}", pool.size());` → muestra cuantos hilos workers hay
- Línea 92: `    println!("Rutas: GET / | GET /health | GET /info | POST /echo");` → muestra las rutas disponibles
- Línea 93: `    println!("Prueba: curl -k https://{}/health", args.addr);` → muestra un comando de prueba con curl
- Línea 95: `    let inicio = Instant::now();` → guarda la hora de inicio para medir la duracion
- Línea 96: `    for stream in listener.incoming() {` → abre el bucle que espera clientes
- Línea 97: `        match stream {` → comprueba si la conexion se acepto bien
- Línea 98: `            Ok(stream) => {` → si llego un cliente, abre este caso
- Línea 99: `                let config = Arc::clone(&config);` → hace otra referencia compartida de la configuración para el hilo
- Línea 100: `                let threads = args.threads;` → copia el número de hilos para mostrarlo después
- Línea 101: `                pool.execute(move || {` → mete el trabajo de este cliente en la cola del pool
- Línea 102: `                    if let Err(e) = http::handle_connection(stream, config, threads) {` → atiende al cliente y detecta si hubo error
- Línea 103: `                        eprintln!("[{}] error en conexión: {e}", hilo_actual());` → imprime el error con el nombre del hilo
- Línea 104: `                    }` → cierra la comprobacion de error
- Línea 105: `                });` → cierra el trabajo enviado al pool
- Línea 106: `            }` → cierra el caso de cliente aceptado
- Línea 107: `            Err(e) => eprintln!("Error al aceptar conexión: {e}"),` → si falla la aceptacion, lo imprime
- Línea 108: `        }` → cierra el match
- Línea 109: `    }` → cierra el bucle de clientes
- Línea 111: `    println!("Servidor apagado después de {:?}", inicio.elapsed());` → imprime cuanto tiempo estuvo levantado el servidor
- Línea 112: `    Ok(())` → termina sin errores
- Línea 113: `}` → cierra la funcion main
- Línea 115: `pub fn hilo_actual() -> String {` → abre la funcion que devuelve el nombre del hilo actual
- Línea 116: `    std::thread::current()` → obtiene el hilo que esta ejecutando
- Línea 117: `        .name()` → pide su nombre
- Línea 118: `        .unwrap_or("desconocido")` → usa la palabra desconocido si no tiene nombre
- Línea 119: `        .to_string()` → convierte el nombre a texto String
- Línea 120: `}` → cierra la funcion hilo_actual

**DI (Francisco):**
> "En `main.rs` primero leemos las opciones, preparamos el certificado y creamos el pool. Despues abrimos el puerto. Fijense en el bucle: cuando llega un cliente no lo atiende ahí mismo, lo mete en la cola con `pool.execute` y sigue aceptando mas."

### `src/pool.rs` (línea por línea)

**EN PANTALLA:** abrir `src/pool.rs`, primero el pool y luego los workers.

- Línea 1: `use std::sync::mpsc::{self, Receiver, Sender};` → trae el canal para enviar trabajos
- Línea 2: `use std::sync::{Arc, Mutex};` → trae Arc y Mutex para compartir la cola
- Línea 3: `use std::thread::{self, JoinHandle};` → trae herramientas para crear hilos
- Línea 5: `type Job = Box<dyn FnOnce() + Send + 'static>;` → define un trabajo como una funcion que se corre una vez en otro hilo
- Línea 7: `pub struct ThreadPool {` → abre la estructura del pool de hilos
- Línea 8: `    workers: Vec<Worker>,` → guarda la lista de hilos workers
- Línea 9: `    sender: Option<Sender<Job>>,` → guarda el emisor de la cola; se quita al apagar
- Línea 10: `}` → cierra la estructura ThreadPool
- Línea 12: `struct Worker {` → abre la estructura de un worker
- Línea 13: `    id: usize,` → guarda el número del worker
- Línea 14: `    thread: Option<JoinHandle<()>>,` → guarda el hilo creado; se quita al apagar
- Línea 15: `}` → cierra la estructura Worker
- Línea 17: `impl ThreadPool {` → abre los métodos del pool
- Línea 18: `    pub fn new(size: usize) -> Self {` → abre el constructor del pool
- Línea 19: `        assert!(size > 0, "el pool debe tener al menos un hilo");` → exige al menos un hilo
- Línea 21: `        let (sender, receiver) = mpsc::channel::<Job>();` → crea la cola de trabajos: sender mete y receiver saca
- Línea 22: `        let receiver = Arc::new(Mutex::new(receiver));` → hace la cola compartible y con Mutex para que solo uno saque a la vez
- Línea 24: `        let workers = (0..size)` → abre la creacion de los números de worker
- Línea 25: `            .map(|id| Worker::new(id, Arc::clone(&receiver)))` → crea un worker por número y le da la misma cola
- Línea 26: `            .collect();` → junta todos los workers en una lista
- Línea 28: `        Self {` → abre el pool que se devuelve
- Línea 29: `            workers,` → mete la lista de workers en el pool
- Línea 30: `            sender: Some(sender),` → mete el emisor de la cola en el pool
- Línea 31: `        }` → cierra el pool que se devuelve
- Línea 32: `    }` → cierra el constructor
- Línea 34: `    pub fn execute<F>(&self, f: F)` → abre el método que recibe un trabajo
- Línea 35: `    where` → abre la lista de requisitos del trabajo
- Línea 36: `        F: FnOnce() + Send + 'static,` → pide una funcion que se pueda enviar a otro hilo
- Línea 37: `    {` → abre el cuerpo de execute
- Línea 38: `        self.sender` → accede al emisor de la cola
- Línea 39: `            .as_ref()` → toma una referencia al emisor sin consumirlo
- Línea 40: `            .expect("la cola ya está cerrada")` → avisa si la cola ya estaba cerrada
- Línea 41: `            .send(Box::new(f))` → mete el trabajo en la cola
- Línea 42: `            .expect("los workers ya se apagaron");` → avisa si los workers ya se apagaron
- Línea 43: `    }` → cierra el método execute
- Línea 45: `    pub fn size(&self) -> usize {` → abre el método que cuenta los hilos
- Línea 46: `        self.workers.len()` → devuelve cuantos workers hay
- Línea 47: `    }` → cierra el método size
- Línea 48: `}` → cierra el impl de ThreadPool
- Línea 50: `impl Drop for ThreadPool {` → abre lo que se hace al apagar el pool
- Línea 51: `    fn drop(&mut self) {` → abre la funcion de apagado
- Línea 52: `        drop(self.sender.take());` → cierra la cola para que los workers salgan del bucle
- Línea 54: `        for worker in &mut self.workers {` → recorre cada worker para esperarlo
- Línea 55: `            if let Some(thread) = worker.thread.take() {` → toma el hilo si todavia esta guardado
- Línea 56: `                if thread.join().is_err() {` → espera a que termine y comprueba si hubo error
- Línea 57: `                    eprintln!("El worker {} terminó con pánico", worker.id);` → avisa si un worker termino con pánico
- Línea 58: `                }` → cierra la comprobacion de error
- Línea 59: `            }` → cierra el if del hilo
- Línea 60: `        }` → cierra el bucle de workers
- Línea 61: `    }` → cierra la funcion drop
- Línea 62: `}` → cierra el impl Drop
- Línea 64: `impl Worker {` → abre los métodos del worker
- Línea 65: `    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {` → abre el constructor del worker con su número y la cola
- Línea 66: `        let thread = thread::Builder::new()` → empieza a construir un hilo
- Línea 67: `            .name(format!("worker-{id}"))` → le pone por nombre worker-0, worker-1 y asi sucesivamente
- Línea 68: `            .spawn(move || loop {` → lanza el hilo con un bucle infinito
- Línea 69: `                let mensaje = receiver.lock().unwrap().recv();` → bloquea hasta sacar un trabajo de la cola
- Línea 71: `                match mensaje {` → mira si llego un trabajo o si la cola se cerro
- Línea 72: `                    Ok(job) => {` → si llego un trabajo, abre este caso
- Línea 73: `                        println!("[worker-{id}] ejecutando trabajo");` → anuncia que va a ejecutar el trabajo
- Línea 74: `                        job();` → ejecuta el trabajo
- Línea 75: `                    }` → cierra el caso de trabajo
- Línea 76: `                    Err(_) => {` → si la cola se cerro, abre este caso
- Línea 77: `                        println!("[worker-{id}] cola cerrada, apagándome");` → anuncia que el worker se apaga
- Línea 78: `                        break;` → sale del bucle infinito
- Línea 79: `                    }` → cierra el caso de cola cerrada
- Línea 80: `                }` → cierra el match
- Línea 81: `            })` → cierra el bucle y la funcion del hilo
- Línea 82: `            .expect("no se pudo crear el hilo worker");` → avisa si no se pudo crear el hilo worker
- Línea 84: `        Self {` → abre el worker que se devuelve
- Línea 85: `            id,` → guarda el número del worker
- Línea 86: `            thread: Some(thread),` → guarda el hilo creado
- Línea 87: `        }` → cierra el worker que se devuelve
- Línea 88: `    }` → cierra el constructor del worker
- Línea 89: `}` → cierra el impl de Worker
- Línea 91: `#[cfg(test)]` → dice que lo siguiente solo se compila en pruebas
- Línea 92: `mod tests {` → abre el modulo de pruebas
- Línea 93: `    use super::*;` → trae lo del archivo para usarlo en las pruebas
- Línea 94: `    use std::sync::atomic::{AtomicUsize, Ordering};` → trae un contador compartido entre hilos
- Línea 95: `    use std::time::Duration;` → trae Duration para las pausas
- Línea 97: `    #[test]` → marca la siguiente funcion como prueba
- Línea 98: `    fn ejecuta_todos_los_trabajos_en_paralelo() {` → abre la prueba de trabajos en paralelo
- Línea 99: `        const N: usize = 8;` → fija 8 trabajos de prueba
- Línea 100: `        let contador = Arc::new(AtomicUsize::new(0));` → crea un contador compartido empezando en cero
- Línea 101: `        let pool = ThreadPool::new(4);` → crea un pool de 4 hilos
- Línea 103: `        for _ in 0..N {` → abre el bucle que envia los 8 trabajos
- Línea 104: `            let contador = Arc::clone(&contador);` → da a cada trabajo su propia referencia al contador
- Línea 105: `            pool.execute(move || {` → envia el trabajo al pool
- Línea 106: `                thread::sleep(Duration::from_millis(10));` → hace una pausa de 10 milisegundos
- Línea 107: `                contador.fetch_add(1, Ordering::SeqCst);` → suma 1 al contador compartido
- Línea 108: `            });` → cierra el trabajo enviado
- Línea 109: `        }` → cierra el bucle de trabajos
- Línea 111: `        drop(pool);` → apaga el pool y espera a que terminen los hilos
- Línea 112: `        assert_eq!(contador.load(Ordering::SeqCst), N);` → comprueba que los 8 trabajos se ejecutaron
- Línea 113: `    }` → cierra la prueba
- Línea 115: `    #[test]` → marca la siguiente funcion como prueba
- Línea 116: `    #[should_panic]` → dice que la prueba debe fallar a propósito
- Línea 117: `    fn tamano_cero_no_es_valido() {` → abre la prueba de tamaño cero
- Línea 118: `        let _ = ThreadPool::new(0);` → intenta crear un pool con cero hilos
- Línea 119: `    }` → cierra la prueba
- Línea 120: `}` → cierra el modulo de pruebas

**DI (Francisco):**
> "Aquí está el multihilo. La cola se crea con `mpsc::channel`; `sender` mete trabajos y `receiver` los saca. El `Mutex` hace que solo un worker saque cada trabajo, y `recv()` deja al worker dormido hasta que llega algo. Al apagar, `drop` cierra la cola y `join()` espera a todos."

## 5:40 - 5:50 · Transición

**DI (Francisco):**
> "Ya tenemos la parte de hilos. Ahora le paso a Yeimar, que explica cómo se cifra la conexión y cómo se responde."

---

# PARTE 2 — Yeimar Gonzalez

## 5:50 - 6:50 · `src/tls.rs` (línea por línea)

**EN PANTALLA:** abrir `src/tls.rs` y seguir el orden de las líneas.

- Línea 1: `use std::fs;` → trae funciones para leer y escribir archivos
- Línea 2: `use std::io::BufReader;` → trae BufReader para leer archivos por partes
- Línea 3: `use std::path::Path;` → trae Path para manejar rutas
- Línea 5: `use rcgen::{CertificateParams, DnType, KeyPair};` → trae lo necesario para crear certificados
- Línea 6: `use rustls::pki_types::{CertificateDer, PrivateKeyDer};` → trae los tipos de certificado y clave privada
- Línea 7: `use rustls::ServerConfig;` → trae la configuración del servidor TLS
- Línea 9: `pub fn build_server_config(` → abre la funcion que prepara la configuración HTTPS
- Línea 10: `    cert_path: Option<&Path>,` → recibe la ruta del certificado si se dio
- Línea 11: `    key_path: Option<&Path>,` → recibe la ruta de la clave si se dio
- Línea 12: `) -> Result<ServerConfig, Box<dyn std::error::Error>> {` → cierra los parámetros y abre la funcion que devuelve config o error
- Línea 13: `    let (cert, key) = match (cert_path, key_path) {` → decide de donde sacar el certificado y la clave
- Línea 14: `        (Some(c), Some(k)) => load_pem(c, k)?,` → si nos dieron archivos, los lee
- Línea 15: `        (None, None) => generate_and_save_self_signed()?,` → si no nos dieron archivos, crea uno autofirmado
- Línea 16: `        _ => unreachable!("Args valida que --cert y --key vayan juntos"),` → caso imposible porque main ya valida que vayan juntos
- Línea 17: `    };` → cierra la decision
- Línea 19: `    let config = ServerConfig::builder()` → empieza a construir la configuración TLS
- Línea 20: `        .with_no_client_auth()` → no pide certificado a los clientes
- Línea 21: `        .with_single_cert(cert, key)?;` → pone el certificado y la clave; falla si no sirven
- Línea 23: `    Ok(config)` → devuelve la configuración lista
- Línea 24: `}` → cierra la funcion build_server_config
- Línea 26: `fn load_pem(` → abre la funcion que lee certificado y clave de archivos
- Línea 27: `    cert_path: &Path,` → recibe la ruta del certificado
- Línea 28: `    key_path: &Path,` → recibe la ruta de la clave
- Línea 29: `) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {` → devuelve los certificados y la clave, o un error
- Línea 30: `    let mut cert_file = BufReader::new(fs::File::open(cert_path)?);` → abre el archivo del certificado para leerlo
- Línea 31: `    let certs: Vec<CertificateDer<'static>> =` → abre la creacion de la lista de certificados leidos
- Línea 32: `        rustls_pemfile::certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;` → lee todos los certificados PEM y falla si hay error
- Línea 34: `    let mut key_file = BufReader::new(fs::File::open(key_path)?);` → abre el archivo de la clave privada
- Línea 35: `    let key = rustls_pemfile::private_key(&mut key_file)?` → lee la clave privada del archivo PEM
- Línea 36: `        .ok_or("no se encontró una clave privada en el archivo")?;` → falla si el archivo no tenia clave privada
- Línea 38: `    println!("Certificado cargado desde {}", cert_path.display());` → avisa desde que archivo se cargo el certificado
- Línea 39: `    Ok((certs, key))` → devuelve los certificados y la clave
- Línea 40: `}` → cierra la funcion load_pem
- Línea 42: `fn generate_and_save_self_signed(` → abre la funcion que crea un certificado de prueba
- Línea 43: `) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {` → devuelve los certificados y la clave, o un error
- Línea 44: `    let mut params = CertificateParams::new(vec!["localhost".to_string()])?;` → prepara un certificado valido para localhost
- Línea 45: `    params` → accede a los datos del certificado para completarlos
- Línea 46: `        .distinguished_name` → accede al nombre identificativo del certificado
- Línea 47: `        .push(DnType::CommonName, "localhost");` → pone localhost como nombre comun
- Línea 49: `    let clave = KeyPair::generate()?;` → crea la clave privada
- Línea 50: `    let certificado = params.self_signed(&clave)?;` → firma el certificado con esa clave
- Línea 52: `    let cert_der = certificado.der().clone();` → guarda el certificado en formato binario
- Línea 53: `    let key_der = PrivateKeyDer::Pkcs8(clave.serialize_der().into());` → guarda la clave en formato binario
- Línea 55: `    fs::create_dir_all("certs")?;` → crea la carpeta certs si no existe
- Línea 56: `    fs::write("certs/cert.pem", certificado.pem())?;` → guarda el certificado en texto PEM
- Línea 57: `    fs::write("certs/key.pem", clave.serialize_pem())?;` → guarda la clave privada en texto PEM
- Línea 58: `    println!("Certificado autofirmado generado en certs/cert.pem y certs/key.pem");` → avisa de los dos archivos que se crearon
- Línea 59: `    println!("Para confiar en él: curl --cacert certs/cert.pem https://localhost:8443/health");` → muestra como confiar en el certificado con curl
- Línea 61: `    Ok((vec![cert_der], key_der))` → devuelve el certificado y la clave ya en memoria
- Línea 62: `}` → cierra la funcion generate_and_save_self_signed

**DI (Yeimar):**
> "Si nos dan certificado y clave, los leemos. Si no, creamos uno autofirmado para localhost y lo guardamos en la carpeta certs. Con eso rustls ya puede cifrar. Por eso la primera vez aparecen dos archivos nuevos."

## 6:50 - 9:10 · `src/http.rs` (línea por línea)

**EN PANTALLA:** abrir `src/http.rs`: primero `handle_connection`, después la lectura de la peticion, las rutas y las pruebas.

- Línea 1: `use std::io::{self, Read, Write};` → trae leer, escribir y el tipo de error de entrada y salida
- Línea 2: `use std::net::TcpStream;` → trae el socket del cliente
- Línea 3: `use std::sync::Arc;` → trae Arc para compartir la configuración TLS
- Línea 4: `use std::time::Duration;` → trae Duration para los tiempos de espera
- Línea 6: `use rustls::{ServerConfig, ServerConnection};` → trae la configuración y la conexion TLS
- Línea 8: `use crate::hilo_actual;` → trae la funcion que da el nombre del hilo
- Línea 10: `#[derive(Debug, PartialEq, Eq)]` → hace que Request se pueda imprimir y comparar en pruebas
- Línea 11: `struct Request {` → abre la estructura con la peticion del cliente
- Línea 12: `    method: String,` → guarda el método, por ejemplo GET o POST
- Línea 13: `    path: String,` → guarda la ruta, por ejemplo /health
- Línea 14: `    #[allow(dead_code)]` → evita avisos por no usar el campo version
- Línea 15: `    version: String,` → guarda la version HTTP
- Línea 16: `    body: Vec<u8>,` → guarda el cuerpo de la peticion en bytes
- Línea 17: `}` → cierra la estructura Request
- Línea 19: `#[derive(Debug, PartialEq, Eq)]` → hace que Response se pueda imprimir y comparar en pruebas
- Línea 20: `struct Response {` → abre la estructura de la respuesta
- Línea 21: `    status: u16,` → guarda el código de estado, por ejemplo 200
- Línea 22: `    content_type: &'static str,` → guarda el tipo de contenido: texto, HTML o JSON
- Línea 23: `    body: Vec<u8>,` → guarda el contenido de la respuesta en bytes
- Línea 24: `}` → cierra la estructura Response
- Línea 26: `impl Response {` → abre los métodos de Response
- Línea 27: `    fn new(status: u16, content_type: &'static str, body: impl Into<Vec<u8>>) -> Self {` → abre el constructor general de respuestas
- Línea 28: `        Self {` → abre la respuesta que se devuelve
- Línea 29: `            status,` → mete el código de estado
- Línea 30: `            content_type,` → mete el tipo de contenido
- Línea 31: `            body: body.into(),` → convierte el cuerpo a bytes
- Línea 32: `        }` → cierra la respuesta devuelta
- Línea 33: `    }` → cierra el constructor general
- Línea 35: `    fn text(status: u16, body: impl AsRef<[u8]>) -> Self {` → abre el constructor de respuestas de texto plano
- Línea 36: `        Self::new(status, "text/plain", body.as_ref().to_vec())` → crea la respuesta indicando text/plain
- Línea 37: `    }` → cierra el constructor de texto
- Línea 39: `    fn html(status: u16, body: &str) -> Self {` → abre el constructor de respuestas HTML
- Línea 40: `        Self::new(status, "text/html", body.as_bytes().to_vec())` → crea la respuesta indicando text/html
- Línea 41: `    }` → cierra el constructor de HTML
- Línea 43: `    fn json(status: u16, body: &str) -> Self {` → abre el constructor de respuestas JSON
- Línea 44: `        Self::new(status, "application/json", body.as_bytes().to_vec())` → crea la respuesta indicando application/json
- Línea 45: `    }` → cierra el constructor de JSON
- Línea 47: `    fn as_bytes(&self) -> Vec<u8> {` → abre la funcion que convierte la respuesta en bytes para enviar
- Línea 48: `        let motivo = match self.status {` → elige el texto que acompana al código de estado
- Línea 49: `            200 => "OK",` → 200 significa OK
- Línea 50: `            404 => "Not Found",` → 404 significa no encontrado
- Línea 51: `            405 => "Method Not Allowed",` → 405 significa método no permitido
- Línea 52: `            _ => "Unknown",` → cualquier otro código se llama desconocido
- Línea 53: `        };` → cierra la eleccion del motivo
- Línea 54: `        let cabeceras = format!(` → abre la creacion de las cabeceras HTTP
- Línea 55: `            "HTTP/1.1 {} {}\r\n\` → escribe la primera línea con la version y el estado
- Línea 56: `             Content-Type: {}; charset=utf-8\r\n\` → escribe el tipo de contenido
- Línea 57: `             Content-Length: {}\r\n\` → escribe cuantos bytes tiene el cuerpo
- Línea 58: `             Connection: close\r\n\` → indica que la conexion se cierra al terminar
- Línea 59: `             Server: servidor-https-rust/{}\r\n\` → escribe el nombre y la version del servidor
- Línea 60: `             \r\n",` → deja una línea en blanco que separa cabeceras y cuerpo
- Línea 61: `            self.status,` → rellena el código de estado
- Línea 62: `            motivo,` → rellena el texto del estado
- Línea 63: `            self.content_type,` → rellena el tipo de contenido
- Línea 64: `            self.body.len(),` → rellena la longitud del cuerpo
- Línea 65: `            env!("CARGO_PKG_VERSION"),` → rellena la version del proyecto
- Línea 66: `        );` → cierra la creacion de cabeceras
- Línea 67: `        let mut bytes = cabeceras.into_bytes();` → convierte las cabeceras a bytes
- Línea 68: `        bytes.extend_from_slice(&self.body);` → añade el cuerpo detras de las cabeceras
- Línea 69: `        bytes` → devuelve la respuesta completa en bytes
- Línea 70: `    }` → cierra la funcion as_bytes
- Línea 71: `}` → cierra el impl de Response
- Línea 73: `pub fn handle_connection(` → abre la funcion que atiende a un cliente
- Línea 74: `    mut socket: TcpStream,` → recibe el socket del cliente
- Línea 75: `    config: Arc<ServerConfig>,` → recibe la configuración TLS compartida
- Línea 76: `    threads: usize,` → recibe el número de hilos para la ruta /info
- Línea 77: `) -> Result<(), Box<dyn std::error::Error>> {` → cierra los parámetros y abre la funcion que puede fallar
- Línea 78: `    let inicio = std::time::Instant::now();` → guarda la hora de llegada del cliente
- Línea 79: `    let peer = socket.peer_addr()?;` → obtiene la direccion del cliente
- Línea 80: `    socket.set_read_timeout(Some(Duration::from_secs(10)))?;` → limita a 10 segundos la espera de lectura
- Línea 81: `    socket.set_write_timeout(Some(Duration::from_secs(10)))?;` → limita a 10 segundos la espera de escritura
- Línea 83: `    println!("[{}] nueva conexión TLS desde {}", hilo_actual(), peer);` → anuncia la conexion con el hilo y la direccion del cliente
- Línea 85: `    let mut conn = ServerConnection::new(config)?;` → crea la sesion TLS con esa configuración
- Línea 87: `    {` → abre un bloque para soltar el stream antes de cerrar TLS
- Línea 88: `        let mut tls = rustls::Stream::new(&mut conn, &mut socket);` → crea un stream que cifra al leer y al escribir
- Línea 89: `        let request = read_request(&mut tls)?;` → lee la peticion del cliente
- Línea 90: `        let response = route(&request, threads);` → elige la respuesta segun la ruta
- Línea 91: `        println!(` → abre el registro de la peticion
- Línea 92: `            "[{}] {} {} -> {}",` → pone el formato del registro: hilo, método, ruta y estado
- Línea 93: `            hilo_actual(),` → rellena el nombre del hilo
- Línea 94: `            request.method,` → rellena el método de la peticion
- Línea 95: `            request.path,` → rellena la ruta de la peticion
- Línea 96: `            response.status` → rellena el código de respuesta
- Línea 97: `        );` → cierra el registro de la peticion
- Línea 98: `        tls.write_all(&response.as_bytes())?;` → envia la respuesta ya cifrada
- Línea 99: `        tls.flush()?;` → asegura que se envien todos los bytes
- Línea 100: `    }` → cierra el bloque y suelta el stream
- Línea 102: `    conn.send_close_notify();` → avisa al cliente de que se cierra la sesion TLS
- Línea 103: `    {` → abre un bloque para vaciar el aviso de cierre
- Línea 104: `        let mut tls = rustls::Stream::new(&mut conn, &mut socket);` → vuelve a crear el stream cifrado
- Línea 105: `        tls.flush()?;` → envia el aviso de cierre
- Línea 106: `    }` → cierra el bloque de cierre
- Línea 108: `    println!(` → abre el mensaje de fin de conexion
- Línea 109: `        "[{}] conexión de {} cerrada en {:?}",` → pone el formato: hilo, cliente y duracion
- Línea 110: `        hilo_actual(),` → rellena el nombre del hilo
- Línea 111: `        peer,` → rellena la direccion del cliente
- Línea 112: `        inicio.elapsed()` → rellena el tiempo que duro la conexion
- Línea 113: `    );` → cierra el mensaje de fin
- Línea 114: `    Ok(())` → termina sin errores
- Línea 115: `}` → cierra la funcion handle_connection
- Línea 117: `fn read_request(stream: &mut impl Read) -> io::Result<Request> {` → abre la funcion que lee e interpreta la peticion
- Línea 118: `    let mut buf = Vec::with_capacity(1024);` → reserva espacio para lo que llegue
- Línea 119: `    let mut trozo = [0_u8; 1024];` → crea un buffer temporal de 1024 bytes
- Línea 120: `    let mut fin_cabeceras = None;` → aun no sabemos donde acaban las cabeceras
- Línea 122: `    while buf.len() < 64 * 1024 {` → lee como maximo hasta 64 KB por seguridad
- Línea 123: `        if let Some(pos) = encontrar(&buf, b"\r\n\r\n") {` → busca el final de las cabeceras
- Línea 124: `            fin_cabeceras = Some(pos + 4);` → guarda esa posicion mas los 4 bytes del separador
- Línea 125: `            break;` → deja de leer cabeceras
- Línea 126: `        }` → cierra la busqueda
- Línea 127: `        let n = stream.read(&mut trozo)?;` → lee un trozo del socket cifrado
- Línea 128: `        if n == 0 {` → si no llego nada, el cliente cerro la conexion
- Línea 129: `            break;` → sale del bucle
- Línea 130: `        }` → cierra la comprobacion
- Línea 131: `        buf.extend_from_slice(&trozo[..n]);` → añade lo leido al buffer
- Línea 132: `    }` → cierra el bucle de lectura de cabeceras
- Línea 134: `    let fin = fin_cabeceras.unwrap_or(buf.len());` → usa el final encontrado o todo lo leido
- Línea 135: `    let cabeceras = String::from_utf8_lossy(&buf[..fin]);` → convierte las cabeceras a texto
- Línea 136: `    let mut lineas = cabeceras.split("\r\n");` → separa las cabeceras por líneas
- Línea 138: `    let mut partes = lineas.next().unwrap_or("").split_whitespace();` → toma la primera línea y la separa en palabras
- Línea 139: `    let method = partes.next().unwrap_or("").to_string();` → guarda el método, por ejemplo GET
- Línea 140: `    let path = partes.next().unwrap_or("/").to_string();` → guarda la ruta, con barra por defecto
- Línea 141: `    let version = partes.next().unwrap_or("HTTP/1.1").to_string();` → guarda la version HTTP
- Línea 143: `    let mut content_length = 0_usize;` → empieza suponiendo que no hay cuerpo
- Línea 144: `    for linea in lineas {` → recorre el resto de cabeceras
- Línea 145: `        if let Some((clave, valor)) = linea.split_once(':') {` → separa cada cabecera en nombre y valor
- Línea 146: `            if clave.eq_ignore_ascii_case("content-length") {` → busca Content-Length sin importar mayusculas
- Línea 147: `                content_length = valor.trim().parse().unwrap_or(0);` → guarda cuantos bytes trae el cuerpo
- Línea 148: `            }` → cierra la comparacion de la cabecera
- Línea 149: `        }` → cierra la separacion de la cabecera
- Línea 150: `    }` → cierra el bucle de cabeceras
- Línea 152: `    while buf.len() < fin + content_length {` → sigue leyendo hasta tener el cuerpo completo
- Línea 153: `        let n = stream.read(&mut trozo)?;` → lee otro trozo del socket
- Línea 154: `        if n == 0 {` → si el cliente cerro, deja de leer
- Línea 155: `            break;` → sale del bucle
- Línea 156: `        }` → cierra la comprobacion
- Línea 157: `        buf.extend_from_slice(&trozo[..n]);` → añade lo leido al buffer
- Línea 158: `    }` → cierra el bucle de lectura del cuerpo
- Línea 159: `    let body = if buf.len() > fin {` → comprueba si hay bytes de cuerpo
- Línea 160: `        buf[fin..fin + content_length.min(buf.len() - fin)].to_vec()` → copia el cuerpo sin pasarse del buffer
- Línea 161: `    } else {` → si no habia bytes de cuerpo
- Línea 162: `        Vec::new()` → deja el cuerpo vacio
- Línea 163: `    };` → cierra la eleccion del cuerpo
- Línea 165: `    Ok(Request {` → abre la peticion que se devuelve
- Línea 166: `        method,` → mete el método leido
- Línea 167: `        path,` → mete la ruta leida
- Línea 168: `        version,` → mete la version leida
- Línea 169: `        body,` → mete el cuerpo leido
- Línea 170: `    })` → cierra la peticion devuelta
- Línea 171: `}` → cierra la funcion read_request
- Línea 173: `fn encontrar(buf: &[u8], patron: &[u8]) -> Option<usize> {` → abre la funcion que busca un patron de bytes
- Línea 174: `    if patron.is_empty() || buf.len() < patron.len() {` → comprueba los casos en los que no puede haber coincidencia
- Línea 175: `        return None;` → devuelve que no se encontro
- Línea 176: `    }` → cierra la comprobacion
- Línea 177: `    buf.windows(patron.len()).position(|w| w == patron)` → recorre bloques del tamaño del patron y devuelve donde coincide
- Línea 178: `}` → cierra la funcion encontrar
- Línea 180: `fn route(req: &Request, threads: usize) -> Response {` → abre la funcion que elige la respuesta
- Línea 181: `    match (req.method.as_str(), req.path.as_str()) {` → compara método y ruta a la vez
- Línea 182: `        ("GET", "/") => Response::html(200, PORTADA),` → GET / devuelve la portada HTML
- Línea 183: `        ("GET", "/health") => Response::json(200, "{\"status\":\"ok\"}"),` → GET /health devuelve el JSON de salud
- Línea 184: `        ("GET", "/info") => Response::json(` → abre la respuesta JSON de informacion
- Línea 185: `            200,` → pone código 200
- Línea 186: `            &format!(` → abre el texto JSON con datos que cambian
- Línea 187: `                "{{\"version\":\"{}\",\"hilos\":{},\"worker\":\"{}\"}}",` → pone la plantilla con version, hilos y worker
- Línea 188: `                env!("CARGO_PKG_VERSION"),` → rellena la version del proyecto
- Línea 189: `                threads,` → rellena el número de hilos
- Línea 190: `                hilo_actual()` → rellena el nombre del worker que responde
- Línea 191: `            ),` → cierra el texto formateado
- Línea 192: `        ),` → cierra la respuesta de /info
- Línea 193: `        ("POST", "/echo") => Response::text(200, &req.body),` → POST /echo devuelve el mismo cuerpo que recibio
- Línea 194: `        ("GET", "/echo") => Response::new(405, "text/plain", "Usa POST con un cuerpo"),` → GET /echo avisa de que hay que usar POST
- Línea 195: `        _ => Response::html(404, "<h1>404</h1><p>Ruta no encontrada</p>"),` → cualquier otra combinacion da 404
- Línea 196: `    }` → cierra el match
- Línea 197: `}` → cierra la funcion route
- Línea 199: `const PORTADA: &str = r#"<!doctype html>` → abre el texto de la portada HTML
- Línea 200: `<html lang="es">` → abre la pagina indicando que esta en español
- Línea 201: `<head><meta charset="utf-8"><title>Servidor HTTPS en Rust</title></head>` → pone la codificacion y el título de la pagina
- Línea 202: `<body>` → abre el cuerpo de la pagina
- Línea 203: `  <h1>servidor-https-rust</h1>` → muestra el título grande
- Línea 204: `  <p>HTTPS con rustls + pool de hilos. Elaborado por Francisco Sevilla y Yeimar Gonzalez.</p>` → muestra la descripcion y los nombres de los autores
- Línea 205: `  <ul>` → abre la lista de enlaces
- Línea 206: `    <li><a href="/health">/health</a></li>` → pone el enlace a /health
- Línea 207: `    <li><a href="/info">/info</a></li>` → pone el enlace a /info
- Línea 208: `  </ul>` → cierra la lista de enlaces
- Línea 209: `</body>` → cierra el cuerpo de la pagina
- Línea 210: `</html>"#;` → cierra la pagina y termina la portada
- Línea 212: `#[cfg(test)]` → dice que lo siguiente solo se compila en pruebas
- Línea 213: `mod tests {` → abre el modulo de pruebas
- Línea 214: `    use super::*;` → trae lo del archivo para probarlo
- Línea 215: `    use std::io::Cursor;` → trae Cursor para simular lecturas en memoria
- Línea 217: `    #[test]` → marca la siguiente funcion como prueba
- Línea 218: `    fn encuentra_el_fin_de_cabeceras() {` → abre la prueba de busqueda del separador
- Línea 219: `        assert_eq!(` → abre la comparacion del primer resultado
- Línea 220: `            encontrar(b"GET / HTTP/1.1\r\n\r\nbody", b"\r\n\r\n"),` → busca el separador en un texto de ejemplo
- Línea 221: `            Some(14)` → espera encontrarlo en la posicion 14
- Línea 222: `        );` → cierra la comparacion
- Línea 223: `        assert_eq!(encontrar(b"sin fin", b"\r\n\r\n"), None);` → comprueba que sin separador devuelve none
- Línea 224: `    }` → cierra la prueba
- Línea 226: `    #[test]` → marca la siguiente funcion como prueba
- Línea 227: `    fn parsea_get_simple() {` → abre la prueba de un GET sencillo
- Línea 228: `        let raw = b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";` → crea una peticion GET de ejemplo
- Línea 229: `        let req = read_request(&mut Cursor::new(raw.to_vec())).unwrap();` → la interpreta con la funcion real
- Línea 230: `        assert_eq!(req.method, "GET");` → comprueba que el método es GET
- Línea 231: `        assert_eq!(req.path, "/health");` → comprueba que la ruta es /health
- Línea 232: `        assert_eq!(req.version, "HTTP/1.1");` → comprueba que la version es HTTP/1.1
- Línea 233: `        assert!(req.body.is_empty());` → comprueba que no hay cuerpo
- Línea 234: `    }` → cierra la prueba
- Línea 236: `    #[test]` → marca la siguiente funcion como prueba
- Línea 237: `    fn parsea_post_con_body() {` → abre la prueba de un POST con cuerpo
- Línea 238: `        let raw = b"POST /echo HTTP/1.1\r\nContent-Length: 5\r\n\r\nhola!";` → crea una peticion POST con 5 bytes de cuerpo
- Línea 239: `        let req = read_request(&mut Cursor::new(raw.to_vec())).unwrap();` → la interpreta con la funcion real
- Línea 240: `        assert_eq!(req.method, "POST");` → comprueba que el método es POST
- Línea 241: `        assert_eq!(req.body, b"hola!");` → comprueba que el cuerpo leido es hola
- Línea 242: `    }` → cierra la prueba
- Línea 244: `    #[test]` → marca la siguiente funcion como prueba
- Línea 245: `    fn enruta_y_serializa() {` → abre la prueba de enrutado y respuesta
- Línea 246: `        let req = Request {` → abre una peticion de prueba
- Línea 247: `            method: "GET".into(),` → pone el método GET
- Línea 248: `            path: "/health".into(),` → pone la ruta /health
- Línea 249: `            version: "HTTP/1.1".into(),` → pone la version HTTP/1.1
- Línea 250: `            body: Vec::new(),` → deja el cuerpo vacio
- Línea 251: `        };` → cierra la peticion de prueba
- Línea 252: `        let res = route(&req, 4);` → obtiene la respuesta con 4 hilos
- Línea 253: `        assert_eq!(res.status, 200);` → comprueba que el código es 200
- Línea 254: `        let bytes = res.as_bytes();` → convierte la respuesta a bytes
- Línea 255: `        let texto = String::from_utf8(bytes).unwrap();` → la convierte a texto para revisarla
- Línea 256: `        assert!(texto.starts_with("HTTP/1.1 200 OK\r\n"));` → comprueba la primera línea de la respuesta
- Línea 257: `        assert!(texto.contains("Content-Length: 15\r\n"));` → comprueba la longitud del cuerpo en las cabeceras
- Línea 258: `    }` → cierra la prueba
- Línea 260: `    #[test]` → marca la siguiente funcion como prueba
- Línea 261: `    fn desconocido_da_404() {` → abre la prueba de ruta desconocida
- Línea 262: `        let req = Request {` → abre una peticion de prueba
- Línea 263: `            method: "GET".into(),` → pone el método GET
- Línea 264: `            path: "/nope".into(),` → pone una ruta que no existe
- Línea 265: `            version: "HTTP/1.1".into(),` → pone la version HTTP/1.1
- Línea 266: `            body: Vec::new(),` → deja el cuerpo vacio
- Línea 267: `        };` → cierra la peticion de prueba
- Línea 268: `        assert_eq!(route(&req, 4).status, 404);` → comprueba que la respuesta es 404
- Línea 269: `    }` → cierra la prueba
- Línea 270: `}` → cierra el modulo de pruebas

**DI (Yeimar):**
> "La conexión TLS envuelve al socket, leemos la peticion y un `match` decide la respuesta. `/info` muestra el nombre del worker que respondió, y `Content-Length` le dice al navegador cuántos bytes tiene el cuerpo. Al final están las pruebas que comprueban todo esto."

## 9:10 - 9:45 · Demostración

**EN PANTALLA:** una terminal con el servidor y otra con los comandos `curl`.

**DI (Yeimar):**
> "Lo ejecutamos con cuatro hilos y probamos las tres rutas."

```powershell
cargo run --release -- --threads 4
curl -k https://localhost:8443/health
curl -k https://localhost:8443/info
curl -k -X POST -d "hola desde rust" https://localhost:8443/echo
```

**DI (Yeimar):**
> "Usamos `-k` porque el certificado es de prueba. En `/info` vemos worker-0, worker-1... eso confirma que el trabajo se reparte entre los hilos."

---

# CIERRE — Ambos

## 9:45 - 10:00 · Créditos

**DI (Francisco):**
> "El código, la investigación y este guion están en GitHub."

**DI (Yeimar):**
> "Somos Francisco Sevilla y Yeimar Gonzalez. Gracias por ver el video."

## Checklist antes de grabar

- [x] Una viñeta por cada línea no vacía, sin omitir llaves, cierres ni pruebas.
- [x] Números de línea tomados de los archivos finales tras `cargo fmt`.
- [x] Nombres reales: Francisco Sevilla y Yeimar Gonzalez.
- [x] Demostración con `cargo run --release -- --threads 4` y las rutas /health, /info y POST /echo.
- [ ] `cargo test` en verde.
- [ ] `python documentacion/generar_docx.py` regenera el docx sin errores.
- [ ] No subir `certs/key.pem`.
