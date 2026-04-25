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
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
        },
        surface: {
          base: '#0B0F19',
          layer: '#111827',
          card: '#1F2937',
          border: '#374151',
        },
        content: {
          primary: '#E5E7EB',
          secondary: '#9CA3AF',
        },
        brand: {
          indigo: '#6366F1',
          cyan: '#06B6D4',
        },
        godot: {
          blue: '#478cbf',
          dark: '#202531',
        }
      },
      backgroundImage: {
        'brand-gradient': 'linear-gradient(135deg, #6366F1 0%, #06B6D4 100%)',
      }
    },
  },
  plugins: [],
}
