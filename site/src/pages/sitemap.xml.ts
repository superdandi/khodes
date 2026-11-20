import type { APIRoute } from 'astro';
import { absoluteSiteUrl } from '../lib/site.mjs';

export const prerender = true;

const PAGES = [
  '/',
  '/downloads/',
  '/installation/',
  '/docs/',
  '/changelog/',
  '/license/',
  '/support/',
];

function escapeXml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

export const GET: APIRoute = () => {
  const entries = PAGES.map(
    (page) => `  <url>\n    <loc>${escapeXml(absoluteSiteUrl(page))}</loc>\n  </url>`,
  ).join('\n');

  const xml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${entries}\n</urlset>\n`;

  return new Response(xml, {
    headers: {
      'Content-Type': 'application/xml; charset=utf-8',
    },
  });
};
