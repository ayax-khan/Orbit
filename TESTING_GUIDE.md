# ORBIT Screen Sharing Testing Guide

This guide will help you set up and test the complete ORBIT screen sharing functionality.

## Prerequisites

- **PostgreSQL** installed and running
- **Node.js** (v18+) and npm
- **Rust** toolchain (for building backend and desktop agent)
- **Chrome browser** (for the extension)

## Step 1: Set Up PostgreSQL Database

1. Create a database for ORBIT:
```sql
CREATE DATABASE orbit;
CREATE USER orbit WITH PASSWORD 'orbit';
GRANT ALL PRIVILEGES ON DATABASE orbit TO orbit;
```

2. The backend will automatically run migrations on startup.

## Step 2: Configure and Run Backend

1. Navigate to the backend directory:
```bash
cd backend
```

2. Copy the example environment file and configure it:
```bash
cp .env.example .env
```

3. Edit `.env` with your settings:
```env
DATABASE_URL=postgres://orbit:orbit@localhost:5432/orbit
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key-here-change-in-production
BIND_ADDRESS=127.0.0.1:8080
```

4. Run the backend:
```bash
cargo run
```

The backend should start on `http://127.0.0.1:8080`

## Step 3: Build and Load Chrome Extension

1. Navigate to the chrome-extension directory:
```bash
cd chrome-extension
```

2. Install dependencies and build:
```bash
npm install
npm run build
```

3. Load the extension in Chrome:
   - Open Chrome and navigate to `chrome://extensions/`
   - Enable "Developer mode" (toggle in top right)
   - Click "Load unpacked"
   - Select the `chrome-extension/dist` folder

4. The ORBIT extension icon should appear in your browser toolbar.

## Step 4: Register and Login

1. Click the ORBIT extension icon to open the popup
2. Click "Need an account? Register"
3. Fill in your details (email, password, full name)
4. After registration, you'll be logged in automatically

## Step 5: Register a Device

1. In the extension popup, click "Register Device"
2. Enter a device name (e.g., "My Desktop PC")
3. Click "Register Device"
4. Copy the **Device Token** that appears - you'll need this for the desktop agent

## Step 6: Configure and Run Desktop Agent

1. Navigate to the desktop-agent directory:
```bash
cd desktop-agent
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` with your settings:
```env
ORBIT_BACKEND_HOST=127.0.0.1:8080
ORBIT_DEVICE_TOKEN=your-device-token-from-step-5
ORBIT_DEVICE_NAME=My Desktop PC
```

4. Run the desktop agent:
```bash
cargo run
```

The agent will:
- Register itself if needed (using ORBIT_USER_TOKEN instead of ORBIT_DEVICE_TOKEN)
- Send heartbeats to stay online
- Poll for active sessions

## Step 7: Test Screen Sharing

1. Make sure the backend is running
2. Make sure the desktop agent is running and shows "Device registered successfully"
3. In the Chrome extension, you should see your device listed with a green "online" indicator
4. Click "Connect" next to your device
5. A new session will be created and the remote screen view will appear
6. You should see the desktop agent's screen being streamed
7. Try moving your mouse and clicking - input should be forwarded to the host

## Troubleshooting

### Backend won't start
- Check PostgreSQL is running: `pg_isready`
- Verify DATABASE_URL in `.env` is correct
- Check the logs for specific error messages

### Extension won't load
- Make sure you built with `npm run build`
- Load the `dist` folder, not the source folder
- Check Chrome extension permissions

### Device not showing as online
- Verify the desktop agent is running
- Check ORBIT_BACKEND_HOST is correct
- Verify ORBIT_DEVICE_TOKEN matches what you got from registration
- Check backend logs for heartbeat requests

### Connection fails
- Ensure both backend and agent are running
- Check WebSocket connection in browser DevTools
- Verify CORS settings in backend config
- Check that session was created successfully

### Screen not showing
- Check browser console for WebRTC errors
- Verify data channel is open
- Check agent logs for capture/encode errors
- Try refreshing the extension popup

## Architecture Overview

```
┌─────────────────┐
│  Chrome Browser │
│  (Extension)    │
└────────┬────────┘
         │ HTTP/WebSocket
         ▼
┌─────────────────┐
│   Backend       │
│   (Axum/Rust)   │
│  - Auth         │
│  - Device Mgmt  │
│  - Sessions     │
│  - Signaling    │
└────────┬────────┘
         │ HTTP/WebSocket
         ▼
┌─────────────────┐
│ Desktop Agent   │
│ (Rust/Windows)  │
│  - Screen Cap   │
│  - Encoding     │
│  - Input Sim    │
│  - WebRTC       │
└─────────────────┘
```

## Next Steps for Production

1. **Security**: Use real cryptographic keys for device authentication
2. **TURN Server**: Set up a TURN server for NAT traversal
3. **Hardware Encoding**: Enable NVENC/QSV for better performance
4. **Compression**: Implement proper video compression (H.264)
5. **Audio**: Add audio streaming support
6. **File Transfer**: Add file transfer capabilities
7. **Multi-monitor**: Support for multiple monitors
