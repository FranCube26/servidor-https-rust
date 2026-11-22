# Guion del video: explicando `servidor-https-rust` línea por línea

**Duración estimada:** 6–7 minutos  
**Estructura:** primero habla **Francisco Sevilla** y después **Yeimar Gonzalez**.  
**Idea:** explicar qué hace cada línea, con palabras sencillas, sin entrar en teoría.

> Convención: **EN PANTALLA** = qué mostrar; **DI** = texto literal a decir.

| Tiempo | Parte | Quien habla |
|---|---|---|
| 0:00 – 0:20 | Introducción | Francisco Sevilla |
| 0:20 – 2:40 | `Cargo.toml`, `main.rs` y `pool.rs` | **Francisco Sevilla** |
| 2:40 – 2:50 | Transición | Francisco → Yeimar |
| 2:50 – 6:10 | `tls.rs`, `http.rs` y demostración | **Yeimar Gonzalez** |
| 6:10 – 6:25 | Cierre | Ambos |

---

# PARTE 1 — Francisco Sevilla

## 0:00 – 0:20 · Introducción

**EN PANTALLA:** título del proyecto con los nombres Francisco Sevilla y Yeimar Gonzalez.

**DI (Francisco):**
> "Hola. Vamos a ver qué hace cada línea de nuestro servidor HTTPS. Yo explico el archivo principal y el pool de hilos; después Yeimar explica TLS y HTTP. Al final lo ejecutamos."

---

## 0:20 – 1:20 · `Cargo.toml` y `main.rs`

**EN PANTALLA:** abrir `Cargo.toml` y luego `src/main.rs`.

- `rustls = "0.23"` → librería para hacer TLS, o sea HTTPS.
- `rustls-pemfile = "2"` → lee los archivos de certificado.
- `rcgen = "0.13"` → crea el certificado de prueba.
- `num_cpus = "1"` → cuenta los procesadores para elegir el número de hilos.

**DI (Francisco):**
> "En `Cargo.toml` solo están esas cuatro librerías. Ahora vamos a `main.rs`, que es donde empieza todo."

**EN PANTALLA:** resaltar estas líneas de `main.rs`:

- `mod http; mod pool; mod tls;` → trae los otros tres archivos del proyecto.
- `rustls::crypto::ring::default_provider().install_default()?;` → enciende el motor de cifrado.
- `let args = Args::parse()?;` → lee lo que escribimos al ejecutar: `--threads`, `--addr`, etc.
- `let config = Arc::new(tls::build_server_config(...)?);` → prepara el certificado una sola vez y lo deja listo para compartir.
- `let pool = pool::ThreadPool::new(args.threads);` → crea los hilos workers.
- `let listener = TcpListener::bind(&args.addr)?;` → abre el puerto 8443 y se queda escuchando.
- `for stream in listener.incoming()` → por cada cliente que llega, entra al bucle.
- `pool.execute(move || { ... });` → mete ese cliente en la cola de trabajo.
- `http::handle_connection(stream, config, threads)` → el worker se encarga de cifrar y responder.

**DI (Francisco):**
> "Lo importante es que el servidor no atiende al cliente en el hilo principal. Solo lo mete en la cola y sigue aceptando más clientes. `move` hace que el cliente viaje al hilo worker."

---

## 1:20 – 2:40 · `pool.rs`, los hilos

**EN PANTALLA:** abrir `src/pool.rs`.

- `type Job = Box<dyn FnOnce() + Send + 'static>;` → define un trabajo: una función que se corre una vez y se puede enviar a otro hilo.
- `let (sender, receiver) = mpsc::channel::<Job>();` → crea la cola: `sender` mete trabajos, `receiver` los saca.
- `let receiver = Arc::new(Mutex::new(receiver));` → la cola se comparte entre los hilos; el `Mutex` hace que solo uno saque cada trabajo.
- `Worker::new(id, Arc::clone(&receiver))` → crea cada hilo y le da la misma cola.
- `.name(format!("worker-{id}"))` → le pone nombre: `worker-0`, `worker-1`, etc.
- `pub fn execute(...)` → recibe una función y la mete en la cola con `send`.
- `let mensaje = receiver.lock().unwrap().recv();` → el hilo espera hasta que haya un trabajo.
- `Ok(job) => job()` → cuando llega, lo ejecuta.
- `Err(_) => break` → si la cola se cerró, el hilo termina.
- `drop(self.sender.take())` → al apagar, cierra la cola.
- `thread.join()` → espera a que los hilos terminen.

**DI (Francisco):**
> "Y ahí está el multihilo: cuatro hilos esperando en la cola. Fíjense en `recv()`: el bloqueo del `Mutex` se suelta antes de ejecutar `job()`, así que los cuatro pueden trabajar al mismo tiempo. Cuando cerramos la cola, todos salen y el programa termina limpio."

---

## 2:40 – 2:50 · Transición

**DI (Francisco):**
> "Ya tenemos la parte de hilos. Ahora le paso a Yeimar, que explica cómo se cifra y cómo se responde."

---

# PARTE 2 — Yeimar Gonzalez

## 2:50 – 3:40 · `tls.rs`, el certificado

**EN PANTALLA:** abrir `src/tls.rs`.

- `build_server_config(cert_path, key_path)` → función que devuelve la configuración HTTPS.
- `(Some(c), Some(k)) => load_pem(c, k)?` → si nos dieron archivos, los cargamos.
- `(None, None) => generate_and_save_self_signed()?` → si no, creamos uno de prueba.
- `CertificateParams::new(vec!["localhost"])` → el certificado es para `localhost`.
- `KeyPair::generate()?` → crea la clave privada.
- `params.self_signed(&clave)?` → firma el certificado con esa clave.
- `fs::write("certs/cert.pem", ...)` → guarda el certificado en un archivo.
- `fs::write("certs/key.pem", ...)` → guarda la clave (ese archivo no se sube a GitHub).
- `ServerConfig::builder().with_no_client_auth().with_single_cert(cert, key)?` → configura rustls con ese certificado.

**DI (Yeimar):**
> "En resumen: si hay certificado lo usamos; si no, creamos uno autofirmado y lo guardamos en la carpeta `certs`. Después rustls queda configurado con él. Por eso la primera vez vemos dos archivos nuevos."

---

## 3:40 – 5:10 · `http.rs`, leer y responder

**EN PANTALLA:** abrir `src/http.rs`, primero `handle_connection`.

- `ServerConnection::new(config)?` → crea una conexión TLS para este cliente.
- `rustls::Stream::new(&mut conn, &mut socket)` → permite leer y escribir normal, pero cifrado.
- `read_request(&mut tls)?` → lee la petición del cliente.
- `route(&request, threads)` → decide qué respuesta toca.
- `tls.write_all(&response.as_bytes())?;` → envía la respuesta ya cifrada.
- `conn.send_close_notify();` → cierra la sesión TLS.

**EN PANTALLA:** bajar a `read_request` y `route`.

- `encontrar(&buf, b"\r\n\r\n")` → busca dónde terminan las cabeceras.
- `let method = partes.next()...` → saca `GET` o `POST`.
- `let path = partes.next()...` → saca la ruta, por ejemplo `/health`.
- `if clave.eq_ignore_ascii_case("content-length")` → lee cuántos bytes trae el cuerpo.
- `("GET", "/health") => Response::json(200, ...)` → responde `{"status":"ok"}`.
- `("GET", "/info") => ...` → responde versión, hilos y worker.
- `("POST", "/echo") => Response::text(200, &req.body)` → devuelve lo que nos mandaron.
- `_ => Response::html(404, ...)` → cualquier otra ruta da 404.
- `format!("Content-Length: {}", self.body.len())` → dice cuántos bytes tiene la respuesta.

**DI (Yeimar):**
> "Es un `match` sencillo: según el método y la ruta, devuelve una cosa u otra. `/info` es el más divertido porque muestra el nombre del worker que respondió. Y `Content-Length` le dice al navegador cuándo terminó la respuesta."

---

## 5:10 – 6:10 · Demostración

**EN PANTALLA:** terminal con `cargo run --release -- --threads 4`, y otra terminal con los `curl`.

**DI (Yeimar):**
> "Lo ejecutamos con cuatro hilos y probamos las rutas."

```powershell
cargo run --release -- --threads 4
curl -k https://localhost:8443/health
curl -k https://localhost:8443/info
curl -k -X POST -d "hola desde rust" https://localhost:8443/echo
```

**DI (Yeimar):**
> "`-k` se usa porque el certificado es de prueba. En `/info` vemos `worker-0`, `worker-1`... eso confirma que el trabajo se reparte. Si repetimos la petición, cambia el worker."

---

# CIERRE — Ambos

## 6:10 – 6:25 · Créditos

**DI (Francisco):**
> "El código, la investigación y este guion están en GitHub."

**DI (Yeimar):**
> "Somos Francisco Sevilla y Yeimar Gonzalez. Gracias por ver el video."

---

## Checklist antes de grabar

- [x] Explicación línea por línea y sin profundizar demasiado.
- [x] Nombres: Francisco Sevilla y Yeimar Gonzalez.
- [ ] `cargo test` en verde.
- [ ] `cargo run --release -- --threads 4` funciona.
- [ ] `curl -k https://localhost:8443/health` responde `{"status":"ok"}`.
- [ ] No subir `certs/key.pem`.
