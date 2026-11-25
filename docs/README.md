# Documentación de SolanoStickers

**🌐 Idioma / Language / Idioma:** **🇪🇸 Español (Principal)** · [🇺🇸 English](#english-documentation-index) · [🇧🇷 Português](#português-índice-da-documentação)

Lee esta guía en orden al comenzar a trabajar con la arquitectura e implementación de SolanoStickers.

## Planificación

| Documento | Propósito |
| --- | --- |
| [MASTER-TODO.md](MASTER-TODO.md) | Lista de verificación de implementación, fase por fase |
| [roadmap.md](roadmap.md) | Cómo se divide el trabajo por etapas |
| [risks.md](risks.md) | Riesgos identificados y mitigaciones adoptadas |

## Producto y requisitos

| Documento | Propósito |
| --- | --- |
| [product/product-brief.md](product/product-brief.md) | Problema, usuarios, alcance y no-objetivos |
| [requirements/functional-requirements.md](requirements/functional-requirements.md) | Qué debe hacer el producto |
| [requirements/non-functional-requirements.md](requirements/non-functional-requirements.md) | Calidad, privacidad, plataformas y rendimiento |
| [requirements/acceptance-criteria.md](requirements/acceptance-criteria.md) | Criterios de aceptación |

## Especificaciones

| Documento | Propósito |
| --- | --- |
| [specifications/platforms.md](specifications/platforms.md) | Límites en Windows, macOS, Chrome y Web |
| [specifications/user-flows.md](specifications/user-flows.md) | Pantallas y flujo principal |
| [specifications/import.md](specifications/import.md) | Portapapeles, archivos y arrastrar/soltar |
| [specifications/myinstants.md](specifications/myinstants.md) | Contrato del proveedor MyInstants |
| [specifications/editors.md](specifications/editors.md) | Reglas de edición de audio y sticker |
| [specifications/data-model.md](specifications/data-model.md) | Proyecto, sticker, audio y asociación |
| [specifications/export.md](specifications/export.md) | WebP, paquete ZIP, MP4/WebM y adaptadores |
| [specifications/whatsapp.md](specifications/whatsapp.md) | Límites oficiales de stickers y notas sobre WhatsApp |
| [specifications/security.md](specifications/security.md) | Validación de URLs, archivos, GIFs y paquetes |
| [specifications/schemas/solanostickers-project-1.0.schema.json](specifications/schemas/solanostickers-project-1.0.schema.json) | Esquema JSON del proyecto portable |

## Arquitectura y decisiones

| Documento | Propósito |
| --- | --- |
| [architecture/overview.md](architecture/overview.md) | Capas, motores y paquetes compartidos |
| [architecture/repository-layout.md](architecture/repository-layout.md) | Estructura del monorepo |
| [decisions/001-product-scope.md](decisions/001-product-scope.md) | Nombre, experiencia y alcance |
| [decisions/002-stack.md](decisions/002-stack.md) | Tauri, Rust, React, Chrome |
| [decisions/003-export-and-whatsapp.md](decisions/003-export-and-whatsapp.md) | Por qué la exportación está desacoplada |
| [decisions/004-extension-boundary.md](decisions/004-extension-boundary.md) | Qué puede y qué no puede hacer la extensión |

## Calidad

| Documento | Propósito |
| --- | --- |
| [qa/test-plan.md](qa/test-plan.md) | Pruebas funcionales, de plataforma y seguridad |

---

<details id="english-documentation-index">
<summary><strong>🇺🇸 English — Documentation Index (Click to expand)</strong></summary>

<br />

Read in this order when starting implementation:

- **Plan:** [MASTER-TODO.md](MASTER-TODO.md) · [roadmap.md](roadmap.md) · [risks.md](risks.md)
- **Product & Requirements:** [product/product-brief.md](product/product-brief.md) · [functional-requirements.md](requirements/functional-requirements.md) · [non-functional-requirements.md](requirements/non-functional-requirements.md) · [acceptance-criteria.md](requirements/acceptance-criteria.md)
- **Specifications:** [platforms.md](specifications/platforms.md) · [user-flows.md](specifications/user-flows.md) · [import.md](specifications/import.md) · [myinstants.md](specifications/myinstants.md) · [editors.md](specifications/editors.md) · [data-model.md](specifications/data-model.md) · [export.md](specifications/export.md) · [whatsapp.md](specifications/whatsapp.md) · [security.md](specifications/security.md) · [solanostickers-project-1.0.schema.json](specifications/schemas/solanostickers-project-1.0.schema.json)
- **Architecture & Decisions:** [architecture/overview.md](architecture/overview.md) · [architecture/repository-layout.md](architecture/repository-layout.md) · [001-product-scope.md](decisions/001-product-scope.md) · [002-stack.md](decisions/002-stack.md) · [003-export-and-whatsapp.md](decisions/003-export-and-whatsapp.md) · [004-extension-boundary.md](decisions/004-extension-boundary.md)
- **QA:** [qa/test-plan.md](qa/test-plan.md)

</details>

<details id="português-índice-da-documentação">
<summary><strong>🇧🇷 Português — Índice da Documentação (Clique para expandir)</strong></summary>

<br />

Leia nesta ordem ao iniciar a implementação:

- **Planejamento:** [MASTER-TODO.md](MASTER-TODO.md) · [roadmap.md](roadmap.md) · [risks.md](risks.md)
- **Produto e Requisitos:** [product/product-brief.md](product/product-brief.md) · [functional-requirements.md](requirements/functional-requirements.md) · [non-functional-requirements.md](requirements/non-functional-requirements.md) · [acceptance-criteria.md](requirements/acceptance-criteria.md)
- **Especificações:** [platforms.md](specifications/platforms.md) · [user-flows.md](specifications/user-flows.md) · [import.md](specifications/import.md) · [myinstants.md](specifications/myinstants.md) · [editors.md](specifications/editors.md) · [data-model.md](specifications/data-model.md) · [export.md](specifications/export.md) · [whatsapp.md](specifications/whatsapp.md) · [security.md](specifications/security.md) · [solanostickers-project-1.0.schema.json](specifications/schemas/solanostickers-project-1.0.schema.json)
- **Arquitetura e Decisões:** [architecture/overview.md](architecture/overview.md) · [architecture/repository-layout.md](architecture/repository-layout.md) · [001-product-scope.md](decisions/001-product-scope.md) · [002-stack.md](decisions/002-stack.md) · [003-export-and-whatsapp.md](decisions/003-export-and-whatsapp.md) · [004-extension-boundary.md](decisions/004-extension-boundary.md)
- **Qualidade (QA):** [qa/test-plan.md](qa/test-plan.md)

</details>
