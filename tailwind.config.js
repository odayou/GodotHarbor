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
          50: '#EFF6FF',
          100: '#DBEAFE',
          200: '#BFDBFE',
          300: '#93C5FD',
          400: '#7AB8E5',
          500: '#478CBF',
          600: '#3A7BA8',
          700: '#2D6A91',
          800: '#1F597A',
          900: '#124863',
          950: '#0A2E42',
        },
        accent: {
          300: '#7ee8f9',
          400: '#6bd7eb',
          500: '#33b5dc',
          600: '#2cb9e3',
        },
        surface: {
          base: '#1a1a1a',
          layer: '#1e1e1e',
          card: '#252526',
          border: '#3c3c3c',
          hover: '#2a2d2e',
          'light-base': '#f8f9fa',
          'light-layer': '#ffffff',
          'light-card': '#ffffff',
          'light-border': '#dee2e6',
          'light-hover': '#e9ecef',
        },
        content: {
          primary: '#d4d4d4',
          secondary: '#9d9d9d',
          muted: '#6e6e6e',
        },
        brand: {
          primary: '#478CBF',
          secondary: '#5A9FD4',
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
      },
      boxShadow: {
        'xs': '0 1px 2px rgba(0,0,0,0.04)',
        'sm': '0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)',
        'md': '0 4px 6px rgba(0,0,0,0.06), 0 2px 4px rgba(0,0,0,0.04)',
        'lg': '0 10px 15px rgba(0,0,0,0.06), 0 4px 6px rgba(0,0,0,0.04)',
        'xl': '0 20px 25px rgba(0,0,0,0.08), 0 8px 10px rgba(0,0,0,0.04)',
        'dialog': '0 20px 60px rgba(0,0,0,0.12)',
        'popover': '0 4px 16px rgba(0,0,0,0.08), 0 0 0 1px rgba(0,0,0,0.04)',
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
