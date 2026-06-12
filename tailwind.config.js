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
          50: '#EFF5FF',
          100: '#DBE8FE',
          200: '#BFDBFE',
          300: '#93B5FD',
          400: '#6090F8',
          500: '#3574F0',
          600: '#2B5FCC',
          700: '#214CA8',
          800: '#183A84',
          900: '#102A60',
          950: '#0A1A3C',
        },
        accent: {
          300: '#7ee8f9',
          400: '#6bd7eb',
          500: '#33b5dc',
          600: '#2cb9e3',
        },
        surface: {
          base: '#1e1e22',
          layer: '#252529',
          card: '#2c2c32',
          border: '#3d3d44',
          hover: '#36363e',
          input: '#252529',
          'light-base': '#f7f8fa',
          'light-layer': '#ffffff',
          'light-card': '#ffffff',
          'light-border': '#d8d8e0',
          'light-hover': '#ecedf2',
          'light-input': '#ffffff',
        },
        content: {
          primary: '#dfdfdf',
          secondary: '#909090',
          muted: '#6b6b6b',
        },
        brand: {
          primary: '#3574F0',
          secondary: '#5A8EF4',
          accent: '#33b5dc',
        },
        status: {
          healthy: '#22c55e',
          warning: '#f59e0b',
          error: '#ef4444',
          info: '#3b82f6',
        },
      },
      backgroundImage: {
        'card-shimmer': 'linear-gradient(135deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.05) 50%, rgba(255,255,255,0) 100%)',
        'brand-gradient': 'linear-gradient(135deg, #3574F0 0%, #5A8EF4 100%)',
      },
      borderRadius: {
        'island': '10px',
        'btn': '6px',
      },
      boxShadow: {
        'xs': '0 1px 2px rgba(0,0,0,0.04)',
        'sm': '0 1px 3px rgba(0,0,0,0.06)',
        'md': '0 2px 6px rgba(0,0,0,0.06)',
        'lg': '0 4px 12px rgba(0,0,0,0.08)',
        'xl': '0 8px 24px rgba(0,0,0,0.10)',
        'dialog': '0 12px 40px rgba(0,0,0,0.16)',
        'popover': '0 2px 8px rgba(0,0,0,0.06), 0 0 0 1px rgba(0,0,0,0.04)',
      },
      animation: {
        'shimmer': 'shimmer 2s ease-in-out infinite',
      },
      keyframes: {
        shimmer: {
          '0%, 100%': { opacity: '0' },
          '50%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
