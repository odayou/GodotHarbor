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
          'light-base': '#f0f4ff',
          'light-layer': '#f8faff',
          'light-card': '#ffffff',
          'light-border': '#dce4f0',
          'light-hover': '#e8f0fe',
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
        status: {
          healthy: '#22c55e',
          warning: '#f59e0b',
          error: '#ef4444',
          info: '#3b82f6',
        },
      },
      backgroundImage: {
        'brand-gradient': 'linear-gradient(135deg, #2563eb 0%, #3b82f6 50%, #60a5fa 100%)',
        'brand-gradient-short': 'linear-gradient(135deg, #2563eb 0%, #3b82f6 100%)',
        'brand-gradient-accent': 'linear-gradient(135deg, #3b82f6 0%, #06b6d4 100%)',
        'sidebar-acrylic': 'linear-gradient(180deg, rgba(255,255,255,0.82) 0%, rgba(240,244,255,0.78) 100%)',
        'header-acrylic': 'linear-gradient(90deg, rgba(255,255,255,0.85) 0%, rgba(248,250,255,0.80) 100%)',
        'card-shimmer': 'linear-gradient(135deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.05) 50%, rgba(255,255,255,0) 100%)',
      },
      boxShadow: {
        'card': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        'elevated': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        'brand-glow': '0 0 20px rgba(55, 102, 190, 0.3)',
        'acrylic': '0 8px 32px rgba(0, 0, 0, 0.06), inset 0 0 0 1px rgba(255, 255, 255, 0.15)',
        'acrylic-dark': '0 8px 32px rgba(0, 0, 0, 0.3), inset 0 0 0 1px rgba(255, 255, 255, 0.05)',
        'stat-card': '0 2px 8px rgba(37, 99, 235, 0.08), 0 1px 2px rgba(0, 0, 0, 0.04)',
        'stat-card-hover': '0 8px 24px rgba(37, 99, 235, 0.12), 0 2px 8px rgba(0, 0, 0, 0.06)',
      },
      backdropBlur: {
        'acrylic': '20px',
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
