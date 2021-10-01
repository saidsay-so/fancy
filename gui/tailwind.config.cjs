const colors = require('tailwindcss/colors')

module.exports = {
  mode: 'jit',
  purge: ['./src/**/*.{html,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        teal: colors.teal,
      },
    },
  },
  plugins: [require('daisyui')],
  daisyui: {
    themes: [
      'dark',
      'light'
    ], 
  }
}
