import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['test/**/*.test.ts'],
    exclude: ['build/**', '.dart_tool/**', 'rust/**', 'lib/**'],
  },
});
