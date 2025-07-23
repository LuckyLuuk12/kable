import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  css: {
    preprocessorOptions: {
      scss: {
        api: 'modern',
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
  // Tauri expects a static build
  build: {
    target: 'esnext'
  }
});
