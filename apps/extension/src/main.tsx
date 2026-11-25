import React from "react";
import { createRoot } from "react-dom/client";
import { detectLocale, type ExportArtifact } from "@solanostickers/core";
import { WebMediaEngine } from "@solanostickers/media-web";
import { MyInstantsProvider } from "@solanostickers/providers";
import { SolanoStickersMinimalApp, type ShellBridge } from "@solanostickers/ui";
import "../../../packages/ui/src/styles.css";

async function downloadBytes(bytes: Uint8Array, fileName: string, mimeType: string): Promise<void> {
  const blob = new Blob([bytes.slice().buffer], { type: mimeType });
  const url = URL.createObjectURL(blob);
  try {
    if (typeof chrome !== "undefined" && chrome.downloads?.download) {
      await chrome.downloads.download({
        url,
        filename: fileName,
        saveAs: false
      });
    } else {
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      a.click();
    }
  } finally {
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
  }
}

const shell: ShellBridge = {
  async saveArtifact(artifact: ExportArtifact) {
    await downloadBytes(artifact.bytes, artifact.fileName, artifact.mimeType);
  },
  async copyArtifactToClipboard(artifact: ExportArtifact) {
    if (navigator.clipboard?.write && typeof ClipboardItem !== "undefined") {
      try {
        const blob = new Blob([artifact.bytes.slice().buffer], { type: artifact.mimeType });
        await navigator.clipboard.write([new ClipboardItem({ [artifact.mimeType]: blob })]);
      } catch {
        // Fallback: user can use the inline Download button or drag the preview video
      }
    }
  },
  async saveProjectZip(zipBytes: Uint8Array, suggestedName: string) {
    await downloadBytes(zipBytes, suggestedName, "application/zip");
  }
};

const engine = new WebMediaEngine();
const providers = [new MyInstantsProvider()];
const locale = detectLocale(navigator.language);

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <SolanoStickersMinimalApp
      engine={engine}
      providers={providers}
      shell={shell}
      initialLocale={locale}
    />
  </React.StrictMode>
);
