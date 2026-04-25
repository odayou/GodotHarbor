/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eef2ff',
          100: '#dce4ff',
          200: '#b9c9ff',
          300: '#8fa8ff',
          400: '#5a7aff',
          500: '#3766be',
          600: '#2a4fa0',
          700: '#7d4ed2',
          800: '#4c32ad',
          900: '#331f8a',
        },
        accent: {
          300: '#7ee8f9',
          400: '#6bd7eb',
          500: '#33b5dc',
          600: '#2cb9e3',
        },
        surface: {
          base: '#050508',
          layer: '#0f1018',
          card: '#1a1b28',
          border: '#2d2d42',
          hover: '#222236',
        },
        content: {
          primary: '#F7FAFC',
          secondary: '#CBD5E0',
          muted: '#718096',
        },
        brand: {
          primary: '#3766be',
          secondary: '#7d4ed2',
          accent: '#33b5dc',
        },
      },
      backgroundImage: {
        'brand-gradient': 'linear-gradient(135deg, #3766be 0%, #7d4ed2 50%, #33b5dc 100%)',
        'brand-gradient-short': 'linear-gradient(135deg, #3766be 0%, #7d4ed2 100%)',
        'brand-gradient-accent': 'linear-gradient(135deg, #7d4ed2 0%, #33b5dc 100%)',
      },
      boxShadow: {
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        'elevated': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        'brand-glow': '0 0 20px rgba(55, 102, 190, 0.3)',
      }
    },
  },
  plugins: [],
}
