# servidor-https-rust

Un servidor HTTPS multihilo en Rust, con pool de hilos y rustls.

Hecho por **Francisco Sevilla** y **Yeimar Gonzalez**.

## Descargar

- Descargar el ZIP: https://github.com/FranCube26/servidor-https-rust/releases/latest
- O clonar el repo:

```powershell
git clone https://github.com/FranCube26/servidor-https-rust.git
```

## Ejecutar

```powershell
cd servidor-https-rust
cargo run --release -- --threads 4
```

En otra terminal:

```powershell
curl -k https://localhost:8443/health
curl -k https://localhost:8443/info
curl -k -X POST -d "hola" https://localhost:8443/echo
```

Todos los comandos están en **`comandos de prueba.txt`**.

## Qué hay aquí

- `src/` — código del servidor (main, pool de hilos, TLS y HTTP)
- `comandos de prueba.txt` — comandos para compilar, probar y subir
- `Cargo.toml` — dependencias

El repo es público: cualquiera puede verlo, descargarlo y ejecutarlo.
