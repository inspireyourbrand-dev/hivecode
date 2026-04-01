/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        hive: {
          cyan: "#3ebaf4",
          magenta: "#df30ff",
          yellow: "#ffe930",
          green: "#00d49b",
          bg: "#0a0a0a",
          "bg-light": "#f8fafc",
          surface: "#1a1a1a",
          "surface-light": "#ffffff",
          border: "#1e3a4f",
          "border-light": "#e2e8f0",
          "text-secondary": "#a0a0a0",
        },
      },
      fontFamily: {
        sans: ["Poppins", "system-ui", "sans-serif"],
        serif: ["Rajdhani", "serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
      animation: {
        "pulse-dot": "pulse-dot 1.5s ease-in-out infinite",
        "slide-in": "slide-in 0.3s ease-out",
        "fade-in": "fade-in 0.2s ease-out",
        "glow-pulse": "glow-pulse 2s ease-in-out infinite",
        "neon-glow": "neon-glow 1.5s ease-in-out infinite",
      },
      keyframes: {
        "pulse-dot": {
          "0%, 100%": { opacity: "0.4" },
          "50%": { opacity: "1" },
        },
        "slide-in": {
          from: { transform: "translateY(8px)", opacity: "0" },
          to: { transform: "translateY(0)", opacity: "1" },
        },
        "fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        "glow-pulse": {
          "0%, 100%": {
            boxShadow: "0 0 5px rgba(62, 186, 244, 0.5), 0 0 10px rgba(62, 186, 244, 0.3)",
          },
          "50%": {
            boxShadow: "0 0 15px rgba(62, 186, 244, 0.8), 0 0 25px rgba(62, 186, 244, 0.5)",
          },
        },
        "neon-glow": {
          "0%, 100%": {
            textShadow: "0 0 10px rgba(62, 186, 244, 0.5), 0 0 20px rgba(223, 48, 255, 0.3)",
          },
          "50%": {
            textShadow: "0 0 20px rgba(62, 186, 244, 0.8), 0 0 30px rgba(223, 48, 255, 0.6)",
          },
        },
      },
    },
  },
  plugins: [],
};
