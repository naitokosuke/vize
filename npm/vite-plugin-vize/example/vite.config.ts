import { defineConfig } from 'vite';
import { vize } from 'vite-plugin-vize';
import Inspect from 'vite-plugin-inspect';

export default defineConfig({
  plugins: [vize(), Inspect()],
});
