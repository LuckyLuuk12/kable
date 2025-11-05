/**
 * We use SvelteKit's static adapter combined with Tauri to build a desktop application.
 * This configuration allows us to prerender the application and disable server-side rendering.
 * Otherwise the window object would not really be available and Tauri would not work properly.
 */
export const prerender = true;
export const ssr = false;
