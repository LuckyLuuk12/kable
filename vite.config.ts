import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    sveltekit({
      // Consult https://svelte.dev/docs/kit/integrations
      // for more information about preprocessors
      preprocess: vitePreprocess(),

      // adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
      // If your environment is not supported, or you settled on a specific environment, switch out the adapter.
      // See https://svelte.dev/docs/kit/adapters for more information about adapters.
      adapter: adapter()
    })
  ],
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern',
        additionalData: (source, filename) => {
          const normalized = filename.replace(/\\/g, '/');

          if (
            normalized.endsWith('/_inject.scss') ||
            normalized.endsWith('/global.scss')
          ) {
            return source;
          }

          return `@use "#lib/styles/_inject.scss" as *;\n${source}`;
        },
        includePaths: [
          'src/lib/styles',
          'node_modules'
        ],
        implementation: 'sass'
      }
    }
  },
  resolve: {
    alias: {
      $lib: './src/lib'
    }
  },
  // Ignore changes in the src-tauri directory to prevent unnecessary reloads and make `npm run dev` work the first time.
  server: {
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  // Tauri expects a static build
  build: {
    target: 'esnext'
  }
});
