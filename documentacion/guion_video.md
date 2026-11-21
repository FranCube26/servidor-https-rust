# Guion del video: explicando `servidor-https-rust`

**Duración estimada:** 9 minutos  
**Estructura:** primero habla **Francisco Sevilla** y después **Yeimar Gonzalez**, sin interrumpirse.  
**Formato:** pantalla compartida (IDE + terminal) con voz en off; cámara opcional en la intro y el cierre.  
**Objetivo:** que quien vea el video entienda cada parte del código y pueda ejecutarlo por su cuenta.

> Convención: **EN PANTALLA** = qué mostrar; **DI** = texto literal a decir; **NOTA** = consejo de grabación.

## Orden del video

| Tiempo | Bloque | Quien habla |
|---|---|---|
| 0:00 – 0:30 | Introducción | Francisco Sevilla |
| 0:30 – 4:20 | Parte 1: arquitectura, `Cargo.toml`, `main.rs` y `pool.rs` | **Francisco Sevilla** |
| 4:20 – 4:35 | Transición | Francisco → Yeimar |
| 4:35 – 8:45 | Parte 2: `tls.rs`, `http.rs`, demo y pruebas | **Yeimar Gonzalez** |
| 8:45 – 9:00 | Cierre y créditos | Ambos |

---

# PARTE 1 — Francisco Sevilla

## 0:00 – 0:30 · Introducción

**EN PANTALLA:** diapositiva con el título `Servidor HTTPS multihilo en Rust`, los nombres **Francisco Sevilla** y **Yeimar Gonzalez**, fecha y logo de Rust.

**DI (Francisco):**
> "Hola, somos Francisco Sevilla y Yeimar Gonzalez. Hoy les vamos a explicar un servidor HTTPS multihilo escrito en Rust: cifra con TLS, reparte conexiones entre varios hilos y responde peticiones HTTP. Yo empiezo explicando la arquitectura y la parte multihilo; después Yeimar se encarga de TLS, del código HTTP y de la demostración en vivo."

**NOTA:** recorta aquí a la ventana del editor con el proyecto abierto.

---

## 0:30 – 1:15 · Arquitectura general

**EN PANTALLA:** dibujo o diagrama simple:

```
TcpListener → TcpStream → ThreadPool → rustls → HTTP/1.1
                            ↓
                    worker-0, worker-1, worker-2, worker-3
```

**DI (Francisco):**
> "Empecemos por el mapa. Un único hilo se queda en `TcpListener::bind` esperando conexiones. Cuando llega una, no la atiende ahí mismo: la mete en una cola. Cuatro hilos workers —o los que digamos por argumento— sacan trabajos de esa cola, hacen el handshake TLS y escriben la respuesta. Así el hilo que acepta nunca se bloquea cifrando, y el número de hilos siempre está acotado."

**DI (Francisco):**
> "Fíjense en la diferencia con un servidor que crea un hilo por conexión: aquí no importa si llegan mil peticiones, seguiremos teniendo cuatro hilos. Eso es *backpressure* natural."

---

## 1:15 – 2:30 · `Cargo.toml` y `main.rs`

**EN PANTALLA:** abrir `Cargo.toml`, luego `src/main.rs` desde la línea 1.

**DI (Francisco):**
> "En `Cargo.toml` tenemos cuatro dependencias. `rustls` es TLS puro en Rust; le quitamos el proveedor por defecto y activamos `ring`, que se compila con el Visual Studio que ya teníamos instalado. `rustls-pemfile` lee certificados en formato PEM. `rcgen` genera el certificado autofirmado para la demo. Y `num_cpus` nos da el número de hilos por defecto."

**DI (Francisco):**
> "En `main.rs`, lo primero es instalar el proveedor criptográfico. rustls no lo elige por nosotros: `ring::default_provider().install_default()`. Si lo olvidamos, el programa falla en cuanto intenta crear una conexión."

**DI (Francisco):**
> "Luego viene `Args::parse`, un parser manual con `env::args` para no añadir otra dependencia. Aceptamos `--addr`, `--threads`, `--cert` y `--key`. Fíjense en la validación: si te pasas solo `--cert` sin `--key`, salimos con error."

**DI (Francisco):**
> "La línea clave es esta: construimos `Arc::new(tls::build_server_config(...))`. `ServerConfig` es inmutable y es `Send + Sync`, así que un solo `Arc` lo comparten todos los hilos. No queremos leer y parsear el certificado una vez por petición."

**DI (Francisco):**
> "Y abajo está el *accept loop*. Por cada `TcpStream` hacemos `Arc::clone` del config y llamamos a `pool.execute` con un closure `move`. El `move` es obligatorio: el stream y el `Arc` tienen que viajar al otro hilo."

---

## 2:30 – 4:20 · `pool.rs` — el corazón multihilo

**EN PANTALLA:** `src/pool.rs`, resaltar `type Job`, `new`, `execute`, `Drop` y el bucle del worker.

**DI (Francisco):**
> "Aquí está la parte más importante de mi explicación. `Job` es un `Box<dyn FnOnce() + Send + 'static>`. Vamos por partes: `FnOnce` porque cada trabajo se ejecuta una sola vez; `Send` porque cruza un canal entre hilos; y `'static` porque el hilo no puede tomar prestados datos que mueran antes."

**DI (Francisco):**
> "En `new` creamos un canal `mpsc`. El receptor se envuelve en `Arc<Mutex<...>>`: el `Arc` lo comparten los cuatro workers y el `Mutex` garantiza que solo uno saque cada trabajo."

**DI (Francisco):**
> "Cada worker se llama `worker-0`, `worker-1`... mediante `thread::Builder::new().name(...)`. Eso parece un detalle cosmético pero nos va a servir en el demo: en los logs veremos qué hilo atendió cada petición."

**DI (Francisco) — pausa y señalar la línea del `let mensaje`:**
> "Este es el detalle que más quiero explicar. Escribimos `let mensaje = receiver.lock().unwrap().recv();`. El guard del `Mutex` se libera al terminar esa instrucción, **antes** de ejecutar `job()` —los temporales de un `let` mueren al final de la sentencia. Si en cambio ejecutáramos el trabajo dentro del mismo `match` sobre `receiver.lock().unwrap().recv()`, el guard seguiría vivo y solo un hilo podría trabajar a la vez. El pool quedaría convertido en un servidor monoproseso con cuatro hilos durmiendo."

**DI (Francisco):**
> "Cuando el canal se cierra, `recv()` devuelve `Err` y el worker rompe el bucle. ¿Quién lo cierra? El `Drop` de `ThreadPool`: `drop(self.sender.take())`. Después hacemos `join()` de cada hilo para no dejar ninguno colgado. Es apagado ordenado: no se pierde trabajo."

**NOTA:** muestra `cargo test` ejecutando `ejecuta_todos_los_trabajos_en_paralelo` si quieres reforzar el punto.

---

## 4:20 – 4:35 · Transición

**EN PANTALLA:** Francisco mira a cámara y luego corte a Yeimar.

**DI (Francisco):**
> "Con eso tenemos la parte multihilo lista. Ahora le paso el turno a Yeimar, que va a explicar cómo se cifra todo eso con rustls y cómo se responde HTTP."

---

# PARTE 2 — Yeimar Gonzalez

## 4:35 – 5:45 · `tls.rs` — certificados y `ServerConfig`

**EN PANTALLA:** `src/tls.rs`, después la carpeta `certs/` con el PEM.

**DI (Yeimar):**
> "Gracias, Francisco. Pasamos a TLS. `build_server_config` tiene dos caminos: si nos pasan `--cert` y `--key`, cargamos los PEM con `rustls_pemfile`; si no, generamos un certificado autofirmado con `rcgen`."

**DI (Yeimar):**
> "En la generación, `CertificateParams::new(vec!["localhost"])` rellena el *Subject Alternative Name*. Eso es importante: los navegadores modernos miran el SAN, no el CommonName, aunque también lo ponemos por compatibilidad. Llamamos a `KeyPair::generate`, firmamos con `params.self_signed(&clave)` y convertimos todo a DER, que es lo que rustls consume."

**DI (Yeimar):**
> "Guardamos los PEM en `certs/`. La clave privada está en `.gitignore` porque nunca debe subirse a GitHub. Para la demo está bien; en producción usaríamos un certificado de Let's Encrypt."

**DI (Yeimar):**
> "Finalmente `ServerConfig::builder().with_no_client_auth().with_single_cert(cert, key)`. `with_no_client_auth` significa que no pedimos certificado al cliente —no hacemos TLS mutuo— y `with_single_cert` instala nuestro par certificado/clave."

---

## 5:45 – 7:15 · `http.rs` — leyendo y respondiendo

**EN PANTALLA:** `src/http.rs`: `read_request`, `route`, `Response::as_bytes` y `handle_connection`.

**DI (Yeimar):**
> "El módulo HTTP está escrito a mano para que se vea el protocolo. `read_request` lee en bloques de 1024 bytes hasta encontrar `\r\n\r\n`, que marca el final de las cabeceras. La función `encontrar` es un `windows().position()` hecho a mano para que no haya magia."

**DI (Yeimar):**
> "Después buscamos la cabecera `Content-Length` —con `eq_ignore_ascii_case`, porque los nombres de cabeceras no distinguen mayúsculas— y leemos exactamente ese número de bytes de cuerpo. Así funciona `POST /echo`: devolvemos el cuerpo tal cual."

**DI (Yeimar):**
> "El enrutador es un `match` sobre `(método, ruta)`. Tenemos la portada, `/health`, `/info`, `/echo` y un 404. Fíjense en `/info`: devuelve la versión, el número de hilos y el nombre del worker que está respondiendo. Ese campo es la prueba de que el pool de Francisco reparte trabajo."

**DI (Yeimar):**
> "`Response::as_bytes` monta las cabeceras con `Content-Length` y `Connection: close`. El `Content-Length` le dice al cliente cuántos bytes leer; el `close` evita keep-alive y hace que el ciclo petición-respuesta sea corto y predecible."

**DI (Yeimar):**
> "Y aquí la línea que conecta todo: `rustls::Stream::new(&mut conn, &mut socket)`. Implementa `Read` y `Write`, cifrando y descifrando por debajo. Por eso el código HTTP es idéntico al de un `TcpStream` sin cifrar. Al terminar llamamos a `send_close_notify`, que es el 'adiós' criptográfico de TLS, y hacemos un último `flush`."

**DI (Yeimar):**
> "Dos protecciones extra: timeout de 10 segundos de lectura y escritura para que un cliente silencioso no ocupe un worker para siempre, y límite de 64 KiB de cabeceras."

---

## 7:15 – 8:25 · Demo en vivo

**EN PANTALLA:** terminal. Ejecutar:

```powershell
cargo run --release -- --threads 4
```

**DI (Yeimar):**
> "Lo ejecutamos. Salen tres líneas: la dirección, los cuatro hilos y las rutas. Se genera el certificado autofirmado en `certs/`."

**EN PANTALLA:** en otra terminal, ejecutar una detrás de otra:

```powershell
curl -k https://localhost:8443/health
curl -k https://localhost:8443/info
curl -k -X POST -d "hola desde rust" https://localhost:8443/echo
curl --cacert certs/cert.pem https://localhost:8443/health
```

**DI (Yeimar):**
> "Usamos `-k` porque el certificado es autofirmado; con `--cacert certs/cert.pem` le decimos a curl que confíe expresamente en nuestro certificado. Fíjense en `/info`: la primera petición la atendió `worker-0`, la segunda `worker-1`. Y en la consola del servidor aparece un `GET /health -> 200` con el nombre del hilo."

**DI (Yeimar):**
> "Si repetimos `/info` diez veces veremos cómo van cambiando los workers. Eso es exactamente lo que queríamos demostrar."

**NOTA:** deja la terminal del servidor visible a la izquierda y curl a la derecha.

---

## 8:25 – 8:45 · Pruebas y cierre técnico

**EN PANTALLA:** `cargo test` y el final de `investigacion.md`.

**DI (Yeimar):**
> "Antes de terminar, las pruebas. `cargo test` comprueba el parser de GET y POST, el enrutado, la serialización de respuestas y que el pool ejecuta los ocho trabajos que le enviamos. También hay un test que verifica que `ThreadPool::new(0)` hace *panic*. Como siguiente paso, si quisiéramos miles de conexiones, migraríamos a Tokio manteniendo rustls. Pero para este trabajo el pool de hilos cumple el requisito y es mucho más fácil de explicar línea por línea."

---

# CIERRE — Ambos

## 8:45 – 9:00 · Créditos

**EN PANTALLA:** diapositiva final con el enlace al repositorio de GitHub y los nombres.

**DI (Francisco):**
> "Eso es todo. El código, la investigación y este guion están en el repositorio de GitHub."

**DI (Yeimar):**
> "Somos Francisco Sevilla y Yeimar Gonzalez. Gracias por ver el video."

---

## Checklist antes de grabar

- [x] Nombres reales en el guion: Francisco Sevilla y Yeimar Gonzalez.
- [ ] `cargo test` en verde.
- [ ] `cargo run --release -- --threads 4` arranca sin errores.
- [ ] `curl -k https://localhost:8443/health` responde `{"status":"ok"}`.
- [ ] Terminal con fuente grande (16–18 pt) y alto contraste.
- [ ] Subir `key.pem` **nunca** (comprobar `.gitignore`).
- [ ] Video y documentos dentro de la carpeta `documentacion/`.
