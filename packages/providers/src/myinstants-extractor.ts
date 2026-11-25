import { SolanoStickersError } from "@solanostickers/core";
import { validateMyInstantsUrl } from "./url-policy";

export interface ExtractedMyInstantsPage {
  title: string | null;
  media_url: string;
}

const PRIMARY_BUTTON_REGEX =
  /id=["']instant-page-button(?:-element)?["'][^>]*?(?:data-url|onclick)=["'][^"']*?(\/media\/sounds\/[A-Za-z0-9._~!$&'()*+,;=:@%/-]+\.(?:mp3|wav|ogg|m4a))/i;
const DOWNLOAD_LINK_REGEX =
  /href=["'](\/media\/sounds\/[A-Za-z0-9._~!$&'()*+,;=:@%/-]+\.(?:mp3|wav|ogg|m4a))["'][^>]*?download/i;
const MEDIA_PATH_REGEX =
  /(?:https:\/\/(?:www\.)?myinstants\.com)?(\/media\/sounds\/[A-Za-z0-9._~!$&'()*+,;=:@%/-]+\.(?:mp3|wav|ogg|m4a))(?:\?[^"'\s<>)]*)?/i;

const OG_TITLE_REGEX = /<meta\s+[^>]*property=["']og:title["'][^>]*content=["']([^"']+)["'][^>]*>/i;
const OG_TITLE_REV_REGEX = /<meta\s+[^>]*content=["']([^"']+)["'][^>]*property=["']og:title["'][^>]*>/i;
const TITLE_TAG_REGEX = /<title[^>]*>([^<]+)<\/title>/i;

function decodeBasicEntities(input: string): string {
  return input
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">");
}

function cleanTitle(rawTitle: string | null): string | null {
  if (!rawTitle) return null;
  const decoded = decodeBasicEntities(rawTitle)
    .replace(/\s*[-|]\s*(?:Myinstants|Bot[oó]n de sonido|Instant Sound Button|Sound Button).*$/i, "")
    .replace(/\s+/g, " ")
    .trim();
  if (!decoded) return null;
  return decoded.slice(0, 120);
}

export function extractMyInstantsFromHtml(html: string): ExtractedMyInstantsPage {
  const mediaMatch =
    html.match(PRIMARY_BUTTON_REGEX) ?? html.match(DOWNLOAD_LINK_REGEX) ?? html.match(MEDIA_PATH_REGEX);
  if (!mediaMatch || !mediaMatch[1]) {
    throw new SolanoStickersError("remote-unavailable", "No MyInstants media URL found in page HTML.");
  }

  const candidateUrl = `https://www.myinstants.com${mediaMatch[1]}`;
  const validated = validateMyInstantsUrl(candidateUrl);

  const ogMatch = html.match(OG_TITLE_REGEX) ?? html.match(OG_TITLE_REV_REGEX);
  const titleMatch = html.match(TITLE_TAG_REGEX);
  const rawTitle = ogMatch?.[1] ?? titleMatch?.[1] ?? null;

  return {
    title: cleanTitle(rawTitle),
    media_url: validated.toString()
  };
}
