import { defineConfig } from 'astro/config';
import { BASE_PATH, SITE_URL } from './src/lib/site.mjs';

export default defineConfig({
  site: SITE_URL,
  base: BASE_PATH,
  output: 'static',
  trailingSlash: 'always',
  build: {
    format: 'directory',
  },
  server: {
    host: true,
  },
});
