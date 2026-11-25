import { describe, expect, it } from "vitest";
import { isVideoFormat, stickerMimeType } from "@solanostickers/core";
import { clipTimeSec, sniffVideoContainer } from "../src";

function box(type: string, brand = ""): Uint8Array {
  const bytes = new Uint8Array(32);
  bytes.set([0, 0, 0, 24], 0);
  bytes.set([...type].map((c) => c.charCodeAt(0)), 4);
  bytes.set([...brand].map((c) => c.charCodeAt(0)), 8);
  return bytes;
}

describe("video sticker sniffing", () => {
  it("recognises MP4, QuickTime and WebM containers", () => {
    expect(sniffVideoContainer(box("ftyp", "isom"))).toBe("mp4");
    expect(sniffVideoContainer(box("ftyp", "mp42"))).toBe("mp4");
    expect(sniffVideoContainer(box("ftyp", "qt  "))).toBe("mov");
    expect(sniffVideoContainer(box("moov"))).toBe("mov");
    const webm = new Uint8Array(32);
    webm.set([0x1a, 0x45, 0xdf, 0xa3], 0);
    expect(sniffVideoContainer(webm)).toBe("webm");
  });

  it("leaves audio-only MP4 and images alone", () => {
    expect(sniffVideoContainer(box("ftyp", "M4A "))).toBeNull();
    const png = new Uint8Array(32);
    png.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0);
    expect(sniffVideoContainer(png)).toBeNull();
    expect(sniffVideoContainer(new Uint8Array(4))).toBeNull();
  });

  it("maps formats and loops short clips", () => {
    expect(isVideoFormat("mov")).toBe(true);
    expect(isVideoFormat("gif")).toBe(false);
    expect(stickerMimeType("mov")).toBe("video/mp4");
    expect(stickerMimeType("png")).toBe("image/png");
    expect(clipTimeSec(2000, 1500, 10_000)).toBeCloseTo(3.5);
    expect(clipTimeSec(0, 4500, 3000)).toBeCloseTo(1.5);
  });
});
