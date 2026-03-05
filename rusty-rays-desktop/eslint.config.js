import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';
import reactX from 'eslint-plugin-react-x';
import reactDom from 'eslint-plugin-react-dom';
import { defineConfig, globalIgnores } from 'eslint/config';

export default defineConfig([
  globalIgnores(['dist']),
  {
    rules: {
      semi: ['error', 'always'],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'after-used',
          varsIgnorePattern: '^_',
          argsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
        },
      ],
      quotes: ['error', 'single', { avoidEscape: true }],
      '@typescript-eslint/restrict-template-expressions': [
        'error',
        {
          allowNumber: true,
        },
      ],
    },
    files: ['src/**/*.{ts,tsx}'],
    ignores: ['src/renderer/retro-ui-lib/**/*'],
    extends: [
      js.configs.recommended,
      tseslint.configs.strictTypeChecked,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    settings: {
      'import/resolver': {
        typescript: {
          project: [
            './tsconfig.node.json',
            './tsconfig.vite.renderer.json',
            './tsconfig.electron.main.json',
          ],
          alwaysTryTypes: true,
        },
      },
    },
    languageOptions: {
      parserOptions: {
        project: [
          './tsconfig.node.json',
          './tsconfig.vite.renderer.json',
          './tsconfig.electron.main.json',
        ],
        tsconfigRootDir: import.meta.dirname,
      },
      ecmaVersion: 2023,
      globals: globals.browser,
    },
  },
]);
