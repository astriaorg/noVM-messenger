import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',  // Accept requests from any host
    port: 3000,        // Set a specific port
    allowedHosts: ['chat.astria-chat.localdev.me'], // Allow this specific host
  },
})


