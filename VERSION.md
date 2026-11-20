# Ficha de versión — Khodes 0.1.0-preview.1

**Estado:** borrador para revisión humana. Esta versión **no está publicada**.
El workflow de release solo crea un *draft*; la publicación se hace a mano desde
la interfaz de GitHub Releases.

- Versión de producto: `0.1.0-preview.1` (`Cargo.toml`)
- Tag previsto: `v0.1.0-preview.1`
- Revisión de origen preparada: `945ddd7952cdc17802e1d81c39ad2b0bb87f4dea`
- Códec de preset: `.khodespreset` v1 (independiente de esta versión)
- Corpus de regresión: `contract/v1/` congelado, 13 fixtures
- Sitio: Astro 7.4.13 estático, sin API en runtime

La revisión de origen debe actualizarse al commit exacto del tag antes de
publicar. Este documento no constituye una aprobación.

---

## 1. Plataformas y arquitecturas

| Plataforma | Runner de CI | Artefacto | Arquitectura |
|---|---|---|---|
| Windows x64 | `windows-latest` | `Khodes-0.1.0-preview.1-windows-x64-unsigned.zip` | x86-64 |
| macOS | `macos-15` | `Khodes-0.1.0-preview.1-macos-unsigned.zip` | Apple silicon (arm64) |
| Linux x64 | `ubuntu-latest` | `Khodes-0.1.0-preview.1-linux-x64-unsigned.zip` | x86-64, glibc |

En x86-64 los bundles se compilan con `--target-cpu v2`. No se publica una
versión para Intel macOS, ARM Windows, Linux musl ni 32 bits. El standalone de
desarrollo no se distribuye.

---

## 2. Requisitos de sistema

- Windows 10 o posterior, x64.
- macOS con Apple silicon, sin versión mínima validada en este entorno.
- Linux x64 con glibc y entorno de escritorio compatible con Slint.

Host CLAP o VST3 con soporte para instrumentos MIDI. No se declara
compatibilidad con ningún DAW concreto: la matriz de hosts sigue pendiente de
validación real (M7-15).

---

## 3. Contenido de cada ZIP

- `Khodes.clap`
- `Khodes.vst3`
- `INSTALL.txt`
- `LICENSE`
- `THIRDPARTY.json`

Fuera de los ZIP, la release incluye `SHA256SUMS` y este `VERSION.md`.

---

## 4. Instalación

Ver `INSTALL.txt`. En resumen: verificar `SHA256SUMS`, extraer ambos formatos
en las rutas habituales del host y volver a escanear plugins.

---

## 5. Limitaciones de seguridad

- Los binarios **no están firmados** y no están notarizados.
- `SHA256SUMS` comprueba integridad, no identidad del publicador.
- La descarga se sirve desde GitHub Releases; no se sirve archivos desde el
  sitio estático.
- El sitio no recoge datos de formularios ni usa analítica propia. Carga el
  badge de atribución de `slint.dev` desde su dominio.

---

## 6. Problemas conocidos y verificaciones pendientes

- La reverb convolucional se ejecuta a 0,7× tiempo real en la CPU de
  desarrollo; no se afirma un gate de CPU cerrado para esta versión.
- No hay validación en REAPER ni Ableton Live; la matriz de DAWs queda abierta.
- El render A/B contra Keys no se ha podido ejecutar: falta render offline de
  Web Audio en este entorno.
- VST3 se valida por generación del bundle y símbolo `GetPluginFactory`, además
  de los validadores de CI; no hay prueba en un host VST3 real documentada.
- El sitio depende de GitHub Releases en tiempo de build. Si la API falla, el
  build falla en lugar de mostrar enlaces incorrectos.

---

## 7. Revisión antes de publicar

- [ ] Confirmar que `Cargo.toml` y el tag coinciden.
- [ ] Actualizar la revisión de origen al commit exacto del tag.
- [ ] Revisar `THIRDPARTY.json` y las licencias de TRUCE y Slint.
- [ ] Confirmar que los validadores de CI pasaron en las tres plataformas.
- [ ] Confirmar que `SHA256SUMS` cubre exactamente tres ZIP.
- [ ] Revisar el *draft* y sus notas, y publicarlo manualmente.
- [ ] Verificar que el sitio desplegado muestra los enlaces y checksums correctos.

Esta lista no está completada. No se crea `APPROVED.txt` sin una decisión
humana explícita.
