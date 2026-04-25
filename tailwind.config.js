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
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
        surface: {
          base: '#0F172A',
          layer: '#1E293B',
          card: '#334155',
          border: '#475569',
        },
        content: {
          primary: '#F8FAFC',
          secondary: '#CBD5E1',
        },
        brand: {
          primary: '#3B82F6',
          secondary: '#10B981',
        },
        godot: {
          blue: '#478cbf',
          dark: '#202531',
        }
      },
      backgroundImage: {
        'brand-gradient': 'linear-gradient(135deg, #3B82F6 0%, #10B981 100%)',
      },
      boxShadow: {
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        'elevated': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
      }
    },
  },
  plugins: [],
}
