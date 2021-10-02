module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: [
    'plugin:@typescript-eslint/recommended',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2019,
    sourceType: 'module',
    tsconfigRootDir: __dirname,
    project: ['./tsconfig.json', './tsconfig.misc.json'],
    extraFileExtensions: ['.svelte'],
  },
  plugins: [
    'svelte3',
    '@typescript-eslint',
  ],
  overrides: [
    {
      files: ['**/*.svelte'],
      processor: 'svelte3/svelte3',
      rules: {
        'import/first': 'off',
        'import/no-duplicates': 'off',
        'import/no-mutable-exports': 'off',
        'import/no-unresolved': 'off',
        'import/prefer-default-export': 'off',
        'import/no-extraneous-dependencies': ['error', {'devDependencies': true}],
      },
    },
  ],
  settings: {
    'svelte3/typescript': true,
  },
};
