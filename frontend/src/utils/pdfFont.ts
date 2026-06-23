import type { jsPDF } from "jspdf";

const fontCache = new Map<string, string>();

async function fetchAsBase64(url: string): Promise<string> {
  if (fontCache.has(url)) return fontCache.get(url)!;
  const res = await fetch(url);
  const buf = await res.arrayBuffer();
  const bytes = new Uint8Array(buf);
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const b64 = btoa(binary);
  fontCache.set(url, b64);
  return b64;
}

const FONT_MAP: Record<string, { url: string; name: string }> = {
  lo: { url: "/fonts/NotoSansLao-Regular.ttf", name: "NotoSansLao" },
  th: { url: "/fonts/NotoSansThai-Regular.ttf", name: "NotoSansThai" },
};

/**
 * Registers a locale-appropriate TTF font with jsPDF.
 * Returns the font name to pass to doc.setFont() and autoTable styles.
 * Falls back to "helvetica" for English (no file fetch needed).
 *
 * Font files must be placed in frontend/public/fonts/:
 *   - NotoSansLao-Regular.ttf  (download from fonts.google.com/noto)
 *   - NotoSansThai-Regular.ttf (download from fonts.google.com/noto)
 */
export async function registerPdfFont(doc: jsPDF, locale: string): Promise<string> {
  const entry = FONT_MAP[locale];
  if (!entry) return "helvetica";

  const b64 = await fetchAsBase64(entry.url);
  doc.addFileToVFS(`${entry.name}.ttf`, b64);
  // Register both styles so setFont(name, 'bold') uses correct glyphs too
  doc.addFont(`${entry.name}.ttf`, entry.name, "normal");
  doc.addFont(`${entry.name}.ttf`, entry.name, "bold");
  return entry.name;
}
