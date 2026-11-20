# Releases y versionado — Khodes

**Estado:** proceso objetivo, basado en Buffer Uppercut.

Khodes usa **GitHub Releases como fuente de verdad** de los binarios y un
**sitio estático** que lee esas releases en tiempo de build. Ningún workflow
publica una release automáticamente: el CI prepara un draft y una persona lo
revisa y publica.

---

## 0. Repositorio y dominio de esta entrega

La fuente de verdad es `superdandi/khodes` y la GitHub Page se sirve en
`https://superdandi.github.io/khodes/`. El release se etiqueta en ese repositorio
y el sitio lee sus releases públicas en tiempo de build mediante
`GITHUB_REPOSITORY=superdandi/khodes`. La decisión y sus consecuencias están en
[ADR 0007](adr/0007-github-owner-and-first-preview-publication.md).

---

## 1. Esquema de versiones

```text
vMAJOR.MINOR.PATCH-preview.N
```

| Ejemplo | Significado |
|---|---|
| `v0.1.0-preview.1` | Primer preview público |
| `v0.1.0-preview.2` | Segundo preview de la misma línea |
| `v0.1.0` | Primera versión estable |
| `v0.2.0` | Nueva funcionalidad compatible |
| `v1.0.0` | Primer estable mayor |

Reglas:

- `MAJOR`: cambios incompatibles de contrato (parámetros, preset, formato).
- `MINOR`: funcionalidad nueva compatible.
- `PATCH`: correcciones compatibles.
- `-preview.N`: preestreno, sin promesa de estabilidad, binarios sin firma.
- La versión de producto es independiente de:
  - `PRESET_VERSION` (códec `.khodespreset`);
  - `DSP_CONTRACT_VERSION` (corpus de audio);
  - versión del sitio.

---

## 2. Artefactos por release

| Plataforma | Archivo |
|---|---|
| Windows x64 | `Khodes-VERSION-windows-x64-unsigned.zip` |
| macOS (Apple silicon) | `Khodes-VERSION-macos-unsigned.zip` |
| Linux x64 | `Khodes-VERSION-linux-x64-unsigned.zip` |
| Integridad | `SHA256SUMS` |
| Licencia | `LICENSE` |
| Instalación | `INSTALL.txt` |
| Notas | cuerpo de la release |

Contenido de cada ZIP:

- `Khodes.clap`;
- `Khodes.vst3`;
- `INSTALL.txt`;
- `LICENSE`;
- avisos de terceros cuando correspondan.

El standalone no se distribuye.

---

## 3. Entradas obligatorias antes del tag

Antes de crear `vVERSION`:

1. **Versión en Cargo.** Actualizar la versión del workspace, incluida
   `-preview.N`, y el lockfile.
2. **`VERSION.md`.** Arquitecturas exactas, requisitos de OS/runtime, revisión
   de fuente, instalación, limitaciones de seguridad y problemas conocidos.
3. **`THIRDPARTY.json`.** Inventario declarado de dependencias directas, con
   `status` y `review` explícitos. Debe revisarse y ampliarse con los crates
   transitivos antes de publicar. `cargo-bundle-licenses` no está disponible en
   el entorno de desarrollo actual, así que el inventario no es automático.
4. **Smoke tests reales.** Registrar pruebas en DAW por plataforma y formato,
   atadas a los artefactos candidatos. Completar [TESTING](TESTING.md) o
   documentar limitaciones aceptadas.
5. **`APPROVED.txt`.** Añadir la línea exacta `vVERSION` solo después de revisar
   todo lo anterior.

El workflow falla si faltan estos archivos. La regla general es no añadir
aprobaciones de relleno para que CI pase.

**Excepción documentada para `v0.1.0-preview.1`:** el titular de la cuenta
`superdandi` autorizó explícitamente el 2026-11-20 la creación de
`APPROVED.txt` y la publicación del primer preview, con esta declaración:

> te autorizo para crear la aprobacion de relleno y para publicar el primer release todo en automatico, documenta exhaustivamente el proceso y todas las deciciones

No es un bypass del gate: `APPROVED.txt` contiene la versión exacta, CI volvió a
validar todos los archivos obligatorios y el owner que aprobó es el mismo que
controla el repositorio y el tag. La revisión legal de `THIRDPARTY.json` sigue
pendiente y se publica como limitación conocida del preview, no como aprobación
legal. Los detalles están en [ADR 0007](adr/0007-github-owner-and-first-preview-publication.md).

### 3.1 Generación de avisos

Cuando `cargo-bundle-licenses 4.2.0` esté disponible:

```sh
cargo install cargo-bundle-licenses --version 4.2.0 --locked
cargo bundle-licenses --format json --output target/THIRDPARTY-audit.json
```

El inventario automático es una ayuda, no una revisión legal. Hasta entonces,
`THIRDPARTY.json` declara `pendiente-de-revision` y el gate solo comprueba que
existe y es JSON válido. La revisión humana sigue siendo obligatoria.

Para este preview se vendorizó `truce-clap 6.3.0` en `vendor/truce-clap/` con un
backport estrecho que emite `CLAP_PARAM_RESCAN_VALUES` tras una carga de estado
y publica el snapshot inicial en `init`; sin él, `clap-validator` falla cuatro
pruebas de reproducibilidad. El cambio, su licencia y las alternativas
descartadas están en
[ADR 0008](adr/0008-truce-clap-state-rescan-backport.md).

---

## 4. Flujo de release

### 4.1 Preparar el commit

```sh
git diff --cached --check
git commit -m "Release preview N"
git push origin main
```

### 4.2 Crear el tag

```sh
git tag -a v0.1.0-preview.1 -m "Khodes 0.1.0-preview.1"
git push origin v0.1.0-preview.1
```

El tag dispara el workflow **release.yml**. También puede iniciarse a mano con
`gh workflow run release.yml -f version=v0.1.0-preview.1`; en ambos casos la
versión se valida con el patrón `vMAJOR.MINOR.PATCH[-preview.N]`.

### 4.3 Qué hace el workflow

1. **Gate:** exige `APPROVED.txt` con la versión exacta, comprueba `Cargo.toml`
   y la presencia de `VERSION.md`, `THIRDPARTY.json`, `LICENSE` e `INSTALL.txt`.
2. Ejecuta doctor, fmt, Clippy, tests y `rt-paranoid`.
3. Compila CLAP y VST3 para Windows x64, macOS Apple silicon y Linux x64; en
   x86-64 usa `--target-cpu v2`.
4. Valida CLAP con `clap-validator` y VST3 con `pluginval` strictness 10, sobre
   los bundles generados.
5. Empaqueta tres ZIP unsigned con `LICENSE`, `INSTALL.txt` y
   `THIRDPARTY.json`; genera `SHA256SUMS` por plataforma.
6. Ensambla los assets y verifica con `tools/verify-release-assets.mjs` que hay
   exactamente tres ZIP y que sus hashes coinciden.
7. Crea un **draft prerelease**. Nunca publica ni sobrescribe una release
   publicada.

### 4.4 Revisar el draft

```sh
gh run list --workflow release.yml --limit 5
gh run view RUN_ID
gh release view v0.1.0-preview.1 --json isDraft,assets,body
gh release download v0.1.0-preview.1 --dir REVIEW_DIRECTORY
```

En un directorio temporal:

```sh
shasum -a 256 -c SHA256SUMS
```

Verificar:

- que los ZIP contienen CLAP y VST3;
- que `INSTALL.txt` es correcto;
- que `LICENSE` y avisos están presentes;
- que las notas mencionan la versión exacta;
- que los checksums coinciden;
- que no hay artefactos internos.

### 4.5 Publicar

```sh
gh release edit v0.1.0-preview.1 --draft=false --prerelease
```

La publicación es manual y deliberada. No se publica desde CI.

### 4.6 Actualizar el sitio

```sh
gh workflow run pages.yml --ref main
gh run list --workflow pages.yml --limit 5
```

Luego verificar en el navegador que la versión nueva aparece y que los enlaces
de descarga funcionan.

### 4.7 Registro de la ejecución real de `v0.1.0-preview.1`

| Momento | Acción | Resultado |
|---|---|---|
| 2026-11-20 13:53 UTC | `gh api POST repos/superdandi/khodes/pages -f build_type=workflow` | Pages habilitado con Actions y URL `https://superdandi.github.io/khodes/` |
| 2026-11-20 13:54 UTC | Push de `main` y tag `v0.1.0-preview.1` | Disparados `CI`, `Pages` y `Release` |
| 2026-11-20 13:54 UTC | Primer `Pages` | Falló: `check-build` exigía descargas cuando aún no había release publicada |
| 2026-11-20 13:57 UTC | Corrección de `check-build.mjs` y push a `main` | `Pages` y `CI` pasaron; sitio desplegado con estado honesto «Sin descargas publicadas» |
| 2026-11-20 14:04 UTC | Primer `Release` | Falló en `Validate CLAP` de Linux: el ZIP de `clap-validator 0.4.1` contiene un `tar.gz`, no el binario; `find` devolvió vacío |
| 2026-11-20 14:0x UTC | Corrección de `release.yml` | Extraer recursivamente los `*.tar.gz` del ZIP antes de buscar el ejecutable; se vuelve a etiquetar la misma versión |

Dos decisiones derivadas de incidentes reales, no hipotéticos:

1. **Estado sin releases.** `RELEASES_SOURCE=github` con una respuesta válida y
   vacía no es un error de API: el sitio debe mostrar «Sin descargas
   publicadas». `check-build.mjs` acepta `id="no-release-title"`, prohíbe mezclarlo
   con botones de descarga y sigue exigiendo checksums cuando hay descargas.
   Un fallo HTTP, JSON inválido o contrato roto sigue fallando el build.
2. **Artefacto anidado del validador.** La URL con checksum verificado de
   `clap-validator 0.4.1` para Linux y macOS entrega un `tar.gz` dentro del ZIP.
   El workflow extrae ahora ambos niveles y registra la ruta del ejecutable.
   No se relajó ningún validador: se corrigió la instalación.

El primer `CI` de `main` fue cancelado por un push posterior (`concurrency`
con `cancel-in-progress: false` en otras workflows, pero el grupo de CI permite
cancelar el run anterior al llegar uno nuevo). El `CI` definitivo del commit
publicado terminó verde con doctor, fmt, Clippy, tests, `rt-paranoid`, DSP
boundary, bundles CLAP/VST3 y smoke host CLAP.

---

## 5. Firma y promoción

### 5.1 Previews

- Sin Authenticode en Windows.
- Sin Developer ID ni notarización en macOS.
- `SHA256SUMS` verifica integridad, no identidad.
- El usuario debe seguir `INSTALL.txt` y usar un proyecto de prueba.

### 5.2 Stable

La promoción a stable es un proceso separado:

1. Acumular evidencia de DAWs y plataformas.
2. Resolver limitaciones conocidas.
3. Configurar firma y notarización.
4. Revisar licencias y avisos.
5. Publicar una versión sin `-preview`.
6. Marcar el release como latest.
7. Actualizar el sitio.

No se “promueve” un preview renombrando el tag. Se crea un release nuevo.

---

## 6. Versionado de contratos

| Contrato | Variable | Regla |
|---|---|---|
| Producto | `CARGO_PKG_VERSION` | Semver + `-preview.N` |
| Preset nativo | `PRESET_VERSION` | Entero; cambia con el layout |
| Corpus DSP | `DSP_CONTRACT_VERSION` | Entero; cambia con el sonido |
| Estado del host | versión del wrapper | La gestiona TRUCE |
| Sitio | metadata de release | No se versiona aparte |

Un cambio de sonido intencional **no** cambia la versión del preset, pero sí
crea un nuevo corpus y se documenta en las notas.

---

## 7. Changelog

Formato recomendado:

```markdown
# Khodes 0.1.0-preview.1

## Sonido
- ...

## Interfaz
- ...

## MIDI
- ...

## Estado y presets
- ...

## Plataformas
- Windows x64, macOS arm64, Linux x64.
- Sin firma.

## Limitaciones conocidas
- ...
```

Reglas:

- Sin HTML no confiable; el sitio escapa el texto.
- Mencionar cambios de sonido y de esquema.
- Mencionar limitaciones aceptadas.
- No afirmar compatibilidad no probada.

---

## 8. Rollback

Si un release tiene un problema crítico:

1. No borrar el tag ni sobrescribir assets publicados.
2. Publicar un release nuevo con la corrección (`preview.N+1`).
3. Si es necesario, marcar el release problemático como prerelease o añadir un
   aviso en las notas.
4. Actualizar el sitio.
5. Documentar el incidente.

Nunca se mueve un tag publicado.

---

## 9. Checklist de release

- [ ] Versión actualizada en Cargo y lockfile.
- [ ] `VERSION.md` revisado.
- [ ] `THIRDPARTY.json` revisado.
- [ ] Smoke tests de DAW registrados.
- [ ] `APPROVED.txt` contiene la versión exacta.
- [ ] `cargo fmt`, Clippy, tests y `rt-paranoid` verdes.
- [ ] CLAP y VST3 compilan.
- [ ] Validadores pasan sobre los bundles del tag.
- [ ] Screenshots pasan.
- [ ] Draft revisado.
- [ ] Checksums verificados.
- [ ] Release publicada manualmente.
- [ ] Sitio actualizado y verificado.
- [ ] Notas de release claras.
