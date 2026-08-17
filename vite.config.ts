import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern',
        additionalData: `
          @use "$lib/styles/_inject.scss" as *;
        `,
        includePaths: [
          'src/lib/styles',
          'node_modules',
          'node_modules/@kablan/clean-ui/scss'
        ],
        // Force Vite to use regular sass instead of sass-embedded
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
