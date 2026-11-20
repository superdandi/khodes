# 0007 — Owner de GitHub, dominio y publicación del primer preview

- **Estado:** Aceptado
- **Fecha:** 2026-11-20
- **Decisores:** titular de la cuenta `superdandi` (autorización explícita) y agente de ejecución
- **Relacionado:** [0005](0005-site-and-release-versioning.md), [RELEASES](../RELEASES.md), [SITE](../SITE.md)

---

## Contexto

El proyecto se desarrolló inicialmente sin remote de Git y con URLs de ejemplo
`https://github.com/khodes/khodes`. Antes de publicar había que decidir:

1. qué cuenta y qué nombre de repositorio serían la fuente de verdad;
2. qué dominio serviría la GitHub Page;
3. quién y con qué alcance autorizaba la creación de `APPROVED.txt`;
4. cómo se publicaría el primer release sin convertir CI en publicador.

La autenticación disponible (`gh auth status`) corresponde a la cuenta
`superdandi` con scopes `repo`, `workflow`, `gist` y `read:org`. El repositorio
`superdandi/khodes` no existía. El usuario confirmó los pasos 1–4 y escribió
literalmente:

> te autorizo para crear la aprobacion de relleno y para publicar el primer release todo en automatico, documenta exhaustivamente el proceso y todas las deciciones

También pidió ser notificado cuando el plugin se pudiera revisar y descargar
desde la página del repo, o antes si hacía falta otra confirmación.

---

## Decisión

### Identidad remota y dominio

- Remote: `https://github.com/superdandi/khodes.git`.
- Repositorio público: `superdandi/khodes`.
- Rama por defecto: `main`.
- Dominio de la GitHub Page: `https://superdandi.github.io/khodes/`.
- `SITE_URL=https://superdandi.github.io` y `SITE_BASE=/khodes` se fijan también
  como variables del repositorio para que el build no dependa solo de los
  valores derivados de `github.repository_owner`.
- Se actualizan `Cargo.toml`, `truce.toml` y los ejemplos de documentación que
  usaban `khodes/khodes`.

### Aprobación del release

- Se crea `APPROVED.txt` con la línea exacta `v0.1.0-preview.1`.
- Esta aprobación **no es un bypass**: es el ejercicio de la autoridad que el
  contrato reserva a una persona. El usuario es el titular de la cuenta y
  otorgó la autorización de forma explícita y por escrito en esta conversación.
- Se registra en `docs/RELEASES.md` quién autorizó, cuándo y qué alcance tuvo.
- El gate de `release.yml` no se relaja: sigue exigiendo `APPROVED.txt`, versión
  exacta, `VERSION.md`, `THIRDPARTY.json`, `LICENSE` e `INSTALL.txt`.

### Publicación

- CI sigue creando **solo un draft**. No se modifica el workflow para publicar.
- La publicación se ejecuta manualmente con
  `gh release edit v0.1.0-preview.1 --draft=false --prerelease`.
- "Todo en automático" se interpreta como **ejecución autónoma de los pasos
  autorizados**, no como eliminación de la revisión humana ni como publicación
  desde CI; el propio contrato `CONTRACTS.md` §9 prohíbe publicar desde CI.
- El sitio se reconstruye tras publicar el release para que los botones de
  descarga aparezcan en `https://superdandi.github.io/khodes/downloads/`.

---

## Consecuencias

### Positivas

- Hay una única fuente de verdad para owner, repo, dominio y descargas.
- La URL canónica del sitio coincide con el repositorio real.
- La aprobación queda trazada a una autorización humana verificable.
- La publicación sigue siendo una acción deliberada y reversible solo mediante
  un release nuevo.

### Negativas

- El inventario de licencias y la matriz de DAWs siguen incompletos; el preview
  se publica con esas limitaciones documentadas, no ocultas.
- macOS no puede probarse en este entorno Linux; los artefactos se validan en CI
  y se entregan como `unsigned`.
- El dominio depende de GitHub Pages y del nombre del repositorio.

### Neutrales

- El código del plugin no cambia por el cambio de owner salvo en metadatos.
- Los contratos de parámetros, MIDI, preset y DSP quedan intactos.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| Crear una organización `khodes` | No existe y el usuario autorizó su cuenta personal |
| Publicar desde `release.yml` | Prohibido por `CONTRACTS.md` §9 y por ADR 0005 |
| Usar un dominio propio | No hay dominio configurado; `/khodes/` es suficiente y verificable |
| Dejar `APPROVED.txt` sin autorización | Convertiría el gate en una falsa aprobación |
| Publicar sin ejecutar validadores | `clap-validator` y `pluginval` son requisito del release |

---

## Implementación

- [x] Crear `superdandi/khodes` con `gh repo create`.
- [x] Configurar remote `origin`.
- [x] Fijar variables `SITE_URL` y `SITE_BASE`.
- [x] Actualizar URLs de propiedad en Cargo.toml, truce.toml y documentación.
- [x] Crear `APPROVED.txt` con `v0.1.0-preview.1`.
- [x] Ejecutar validadores y corregir fallos antes de publicar.
- [x] Publicar manualmente el draft y verificar la GitHub Page.

---

## Referencias

- [`CONTRACTS.md`](../CONTRACTS.md) §9 y §10.
- [`RELEASES.md`](../RELEASES.md).
- [`SITE.md`](../SITE.md).
- [ADR 0005](0005-site-and-release-versioning.md).
