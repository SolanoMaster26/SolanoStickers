# Avisos de Terceros y Licencias

**🌐 Idioma / Language / Idioma:** **🇪🇸 Español (Principal)** · [🇺🇸 English](#english-third-party-notices) · [🇧🇷 Português](#português-avisos-de-terceiros)

SolanoStickers se publica bajo licencia doble **MIT OR Apache-2.0** (ver [LICENSE-MIT](../../LICENSE-MIT) y [LICENSE-APACHE](../../LICENSE-APACHE)).

Este documento registra el software de terceros, códecs y bibliotecas incluidas o utilizadas por SolanoStickers en sus versiones de escritorio, extensión de navegador y web.

## Política de medios en Desktop y Web

- **Codificación local sin binarios externos:** Desktop, Extensión y Web renderizan mediante WebCodecs (`VideoEncoder` / `AudioEncoder`) y `Canvas` + `MediaRecorder` usando los códecs nativos del sistema operativo / WebView (Windows Media Foundation, Apple VideoToolbox o códecs del navegador), junto con muxers puros en TypeScript (`mp4-muxer` y `webm-muxer`).
- **Nota histórica (`crates/video-engine`):** El crate auxiliar conserva utilidades y pruebas unitarias de validación de rutas para invocar FFmpeg LGPLv2.1+ como sidecar en caso de necesitarse en el futuro, pero la app de escritorio no expone ni ejecuta `ffmpeg`.

## Crates de Rust (`crates/*` y `apps/desktop/src-tauri`)

| Crate | Licencia | Propósito |
| --- | --- | --- |
| `serde` / `serde_json` | MIT OR Apache-2.0 | Serialización de manifiestos JSON |
| `thiserror` | MIT OR Apache-2.0 | Tipos de error estructurados |
| `image` | MIT OR Apache-2.0 | Decodificación y redimensionado de PNG, JPEG y WebP |
| `gif` | MIT OR Apache-2.0 | Inspección y decodificación de GIF animados |
| `webp` | MIT OR Apache-2.0 | Codificación WebP estática con alfa |
| `symphonia` | MPL-2.0 | Lectura/decodificación de audio en Rust puro (MP3, WAV, OGG, AAC) |
| `zip` | MIT | Lectura y escritura de proyectos `.solanostickers.zip` |
| `tauri` | MIT OR Apache-2.0 | Contenedor nativo de aplicación de escritorio |
| `reqwest` | MIT OR Apache-2.0 | Cliente HTTPS con lista permitida para MyInstants |
| `url` | MIT OR Apache-2.0 | Análisis y validación estricta de URLs |

## Paquetes TypeScript y Navegador (`packages/*`, `apps/extension`, `apps/web`)

| Paquete | Licencia | Propósito |
| --- | --- | --- |
| `react` / `react-dom` | MIT | Componentes de interfaz compartidos |
| `fflate` | MIT | Compresión/descompresión ZIP en JS puro |
| `mp4-muxer` | MIT | Multiplexado MP4 ISO-BMFF para WebCodecs |
| `webm-muxer` | MIT | Multiplexado WebM VP9/Opus para WebCodecs |
| `vitest` | MIT | Ejecutor de pruebas unitarias y de integración |

## Aviso sobre derechos de contenido

Los sonidos obtenidos desde MyInstants pertenecen a sus respectivos autores. SolanoStickers no otorga ninguna licencia de redistribución sobre los sonidos descargados.

---

<details id="english-third-party-notices">
<summary><strong>🇺🇸 English — Third-Party Notices (Click to expand)</strong></summary>

<br />

SolanoStickers itself is dual-licensed under **MIT OR Apache-2.0** (see [LICENSE-MIT](../../LICENSE-MIT) and [LICENSE-APACHE](../../LICENSE-APACHE)).

- **Local Media Policy:** Desktop, Extension, and Web encode video/audio locally via WebCodecs / `MediaRecorder` using OS/WebView codecs and pure-TS muxers (`mp4-muxer`, `webm-muxer`) with zero external FFmpeg binary bundled. `crates/video-engine` remains in the workspace with path-validation utilities and unit tests, but is not exposed by the desktop IPC.
- **Rust Crates:** `serde` / `serde_json`, `thiserror`, `image`, `gif`, `webp`, `tauri`, `reqwest`, `url` (`MIT OR Apache-2.0`); `zip` (`MIT`); `symphonia` (`MPL-2.0`).
- **TypeScript Packages:** `react`, `react-dom`, `fflate`, `mp4-muxer`, `webm-muxer`, `vitest` (`MIT`).
- **Content Rights:** Sounds fetched from MyInstants remain the property of their respective owners.

</details>

<details id="português-avisos-de-terceiros">
<summary><strong>🇧🇷 Português — Avisos de Terceiros e Licenças (Clique para expandir)</strong></summary>

<br />

O SolanoStickers é distribuído sob licença dupla **MIT OR Apache-2.0** (veja [LICENSE-MIT](../../LICENSE-MIT) e [LICENSE-APACHE](../../LICENSE-APACHE)).

- **Política de Mídia Local:** Desktop, Extensão e Web codificam áudio e vídeo localmente via WebCodecs / `MediaRecorder` usando codecs do sistema operacional/WebView e muxers em TypeScript puro (`mp4-muxer`, `webm-muxer`), sem binário FFmpeg embutido. `crates/video-engine` permanece no workspace com utilitários e testes unitários de validação, sem exposição no IPC de desktop.
- **Crates Rust:** `serde` / `serde_json`, `thiserror`, `image`, `gif`, `webp`, `tauri`, `reqwest`, `url` (`MIT OR Apache-2.0`); `zip` (`MIT`); `symphonia` (`MPL-2.0`).
- **Pacotes TypeScript:** `react`, `react-dom`, `fflate`, `mp4-muxer`, `webm-muxer`, `vitest` (`MIT`).
- **Direitos de Conteúdo:** Os sons obtidos do MyInstants permanecem propriedade de seus respectivos autores.

</details>
