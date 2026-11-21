# Investigación: servidor HTTPS multihilo en Rust

**Integrantes:** Francisco Sevilla · Yeimar Gonzalez  
**Fecha:** 21 de noviembre de 2026  
**Entrega:** domingo 22 de noviembre de 2026

---

## 1. Introducción y objetivo

El objetivo del trabajo es implementar un servidor HTTPS capaz de atender varias peticiones **al mismo tiempo**, explicar su funcionamiento en un video y documentar las decisiones técnicas. Se usa Rust por su combinación de rendimiento, seguridad de memoria sin *garbage collector* y un modelo de concurrencia basado en el sistema de tipos (`Send`/`Sync`).

El resultado es `servidor-https-rust`: un servidor que escucha en `127.0.0.1:8443`, termina la conexión TLS con **rustls**, reparte cada conexión a un **pool de hilos** y responde peticiones HTTP/1.1 simples.

---

## 2. HTTPS y TLS

HTTPS es HTTP sobre TLS. TLS proporciona tres garantías:

1. **Confidencialidad:** los bytes viajan cifrados (AES-GCM / ChaCha20-Poly1305).
2. **Integridad:** cualquier modificación en tránsito se detecta mediante AEAD.
3. **Autenticación:** el cliente verifica que el certificado del servidor corresponde al dominio y fue emitido por una autoridad en la que confía.

### 2.1 Handshake de TLS 1.3

En TLS 1.3 (RFC 8446) el intercambio típico es de **1 RTT**:

```
Cliente                                   Servidor
  |-- ClientHello (SNI, key_share, cipher suites) -->|
  |                                                  |  elige parámetros
  |<-- ServerHello (key_share) + certificado cifrado --|
  |<-- Finished -------------------------------------|
  |-- Finished ------------------------------------->|
  |=========== datos cifrados (HTTP) =================|
```

El certificado viaja cifrado porque deriva de los *key shares* intercambiados. La reanudación de sesión permite 0-RTT conPSK, aunque conviene desactivarla en endpoints no idempotentes por riesgo de reproducción (*replay*).

En este proyecto se habilita TLS 1.2 además de 1.3 (`feature = "tls12"`) para poder probar con clientes más antiguos.

### 2.2 ¿Por qué rustls y no OpenSSL?

| Criterio | rustls | OpenSSL / native-tls |
|---|---|---|
| Lenguaje | Rust puro | C (FFI) |
| Seguridad de memoria | garantizada por el compilador | historial de CVEs de parseo |
| Superficie de ataque | pequeña, módulos auditados | grande |
| API | tipada, orientada a estados | C API + envoltorios |
| Proveedor crypto | intercambiable (`ring`, `aws-lc-rs`) | fijo |

`rustls` no elige proveedor criptográfico por sí solo: en la versión 0.23 hay que instalarlo explícitamente con `rustls::crypto::ring::default_provider().install_default()`. Nosotros elegimos `ring` para que coincida con `rcgen` y porque se compila con el compilador de MSVC ya presente, sin necesitar `cmake`.

La configuración del servidor se construye **una sola vez**:

```rust
let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(cert, key)?;
```

`ServerConfig` es `Send + Sync`, por lo que se comparte entre hilos con `Arc<ServerConfig>`. `ServerConnection` es el estado de **una** conexión; no se comparte.

---

## 3. Concurrencia en Rust

### 3.1 Hilos del sistema operativo

`std::thread::spawn` crea un hilo real del SO. Rust evita las *data races* en tiempo de compilación:

* `Send`: el tipo se puede transferir entre hilos.
* `Sync`: el tipo se puede compartir por referencia (`&T` es `Send` si `T: Sync`).

Para compartir datos usamos:

* `Arc<T>`: conteo de referencias atómico, varias propiedades del mismo valor.
* `Mutex<T>`: exclusión mutua; `lock()` devuelve un guard que libera al salir del ámbito (RAII).
* `mpsc::channel`: canal de múltiples productores y un consumidor.

### 3.2 Tres modelos posibles

| Modelo | Ventaja | Inconveniente |
|---|---|---|
| Un hilo por conexión | Muy simple de explicar | Sin límite: miles de conexiones → miles de hilos y agotamiento de memoria |
| **Pool de hilos fijo** | Número acotado de hilos, backpressure natural | Un worker bloqueado reduce capacidad |
| Async (Tokio + axum) | Miles de conexiones con pocos hilos | Código más abstracto; bugs de cancelación y `Send` futures |

Elegimos el **pool de hilos** porque cumple literalmente el requisito "multihilo", es acotado y didáctico: se ve claramente dónde se crean los hilos, dónde se encola el trabajo y cómo se apagan.

### 3.3 Implementación del pool

```rust
type Job = Box<dyn FnOnce() + Send + 'static>;

let (sender, receiver) = mpsc::channel::<Job>();
let receiver = Arc::new(Mutex::new(receiver));
```

Cada worker ejecuta:

```rust
loop {
    let mensaje = receiver.lock().unwrap().recv(); // guard se suelta aquí
    match mensaje {
        Ok(job) => job(),
        Err(_) => break, // el emisor se cerró
    }
}
```

Detalle crítico: el guard del `Mutex` se destruye al terminar la instrucción `let`, **antes** de ejecutar `job()`. Si se mantuviera durante la ejecución, solo un worker podría trabajar a la vez y el pool sería inútil.

El apagado ordenado está en `Drop`: `drop(self.sender.take())` cierra el canal, los `recv()` devuelven `Err`, los workers salen y `join()` espera a todos. Así no se pierde trabajo ni quedan hilos huérfanos.

---

## 4. HTTP/1.1 mínimo

No usamos un framework HTTP para que el video pueda explicar el protocolo línea a línea.

* La petición se lee hasta `\r\n\r\n` (fin de cabeceras) y luego se consumen exactamente `Content-Length` bytes de cuerpo.
* La respuesta lleva `Content-Type`, `Content-Length`, `Connection: close` y un motivo (`OK`, `Not Found`, `Method Not Allowed`).
* `Connection: close` evita el keep-alive: después de responder se cierra la sesión TLS y el socket. Esto simplifica el bucle de lectura, a costa de una conexión nueva por petición.
* Las rutas son `/`, `/health`, `/info` y `POST /echo`; cualquier otra devuelve 404.

`rustls::Stream<'a, C, S>` implementa `Read`/`Write` cifrando y descifrando transparentemente, de modo que el código HTTP no necesita saber que hay TLS debajo. Es el patrón *decorator* aplicado a E/S.

---

## 5. Certificados X.509

Un certificado TLS contiene: clave pública, identidad (Subject), **Subject Alternative Name (SAN)**, emisor, validez y firma. Los clientes modernos verifican el **SAN**, no el `CommonName`, por eso `rcgen` rellena SAN con `localhost`.

* **PEM**: base64 con cabeceras `-----BEGIN CERTIFICATE-----`; legible y fácil de copiar.
* **DER**: binario; es lo que consume `rustls`.
* `rustls-pemfile` convierte PEM → `CertificateDer` / `PrivateKeyDer`.

Para la demo generamos un certificado **autofirmado**: sirve para cifrar y para que `curl -k` funcione, pero ningún navegador lo confiará sin importarlo. En producción se usaría un certificado de una CA (p. ej. Let's Encrypt) y se configuraría la renovación automática.

---

## 6. Diseño del programa

| Módulo | Responsabilidad |
|---|---|
| `main.rs` | Instala proveedor crypto, parsea argumentos, construye `Arc<ServerConfig>`, acepta conexiones y las envía al pool |
| `pool.rs` | Cola de trabajos, N workers, apagado ordenado |
| `tls.rs` | Carga PEM o genera autofirmado con `rcgen`, construye `ServerConfig` |
| `http.rs` | `read_request`, `route`, `Response::as_bytes`, `handle_connection` |

Flujo de una petición:

```
accept → pool.execute(closure) → worker → ServerConnection (handshake)
       → read_request → route → write_all → send_close_notify → log
```

Decisiones destacadas:

* **Un `Arc<ServerConfig>` compartido**, no uno por conexión (ahorra memoria y parseo del certificado).
* **Timeouts de 10 s** de lectura/escritura para que un cliente lento no secuestre un worker.
* **Límite de 64 KiB** de cabeceras como mitigación básica ante peticiones maliciosas.
* **Logs con nombre de hilo** (`worker-0`, `worker-1`…) para poder demostrar en el video que el trabajo se reparte.

---

## 7. Verificación

Se incluyen pruebas unitarias (`cargo test`) para:

* detección del fin de cabeceras `\r\n\r\n`;
* parseo de `GET` sin cuerpo;
* parseo de `POST` con `Content-Length`;
* enrutado y serialización de `200`/`404`;
* ejecución de todos los trabajos por el pool y rechazo de `size = 0`.

Prueba manual con `curl`:

```powershell
curl -k https://localhost:8443/info
```

Salida esperada: `{"version":"0.1.0","hilos":4,"worker":"worker-2"}` — el campo `worker` cambia entre peticiones, lo que confirma el reparto entre hilos.

---

## 8. Conclusiones

Se cumplió el objetivo: un servidor HTTPS funcional, concurrente con un número acotado de hilos y sin dependencias de OpenSSL. El patrón de pool es fácil de auditar y explicar; para un caso real con muchas conexiones simultáneas, el siguiente paso sería migrar a Tokio/hyper manteniendo rustls como backend TLS.

---

## 9. Referencias

1. Rescorla, E. *The Transport Layer Security (TLS) Protocol Version 1.3*, RFC 8446. https://www.rfc-editor.org/rfc/rfc8446
2. Fielding, R. et al. *HTTP/1.1 Semantics*, RFC 9110. https://www.rfc-editor.org/rfc/rfc9110
3. Kémon, T. et al. *HTTP/1.1 Message Syntax and Routing*, RFC 9112. https://www.rfc-editor.org/rfc/rfc9112
4. The Rust Programming Language, cap. 16 y 20 (*Fearless Concurrency* y *Implementing a Multithreaded Web Server*). https://doc.rust-lang.org/book/
5. rustls documentation. https://docs.rs/rustls/latest/rustls/
6. rcgen documentation. https://docs.rs/rcgen/latest/rcgen/
7. ring cryptography crate. https://github.com/briansmith/ring
8. Cooper, D. et al. *Internet X.509 Public Key Infrastructure*, RFC 5280. https://www.rfc-editor.org/rfc/rfc5280
