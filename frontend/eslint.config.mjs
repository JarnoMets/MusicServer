import pluginVue from 'eslint-plugin-vue'
import tseslint from 'typescript-eslint'
import vueParser from 'vue-eslint-parser'

export default [
  // Global ignores
  {
    ignores: ['dist', 'node_modules', '*.config.js', 'public']
  },
  // Recommended for general JS/TS
  ...tseslint.configs.recommended,
  // Recommended for Vue 3
  ...pluginVue.configs['flat/essential'],
  // Vue and TypeScript integration
  {
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        ecmaVersion: 'latest',
        sourceType: 'module'
      }
    }
  },
  // Custom rules for both .vue and .ts files
  {
    files: ['**/*.vue', '**/*.ts'],
    rules: {
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      'no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-empty-object-type': 'warn'
    }
  }
]
