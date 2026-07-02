# ORBIT

Online Remote Binary Interaction Technology

A complete remote desktop solution enabling screen sharing and remote control between users across different locations.

## Features

### Extension-to-Extension Screen Sharing
- **Peer-to-Peer Connection**: Users can connect directly via Chrome extensions without requiring desktop agents
- **Screen Sharing with Audio**: Share your screen with system audio automatically included
- **Remote Control**: Full mouse and keyboard input forwarding for remote control
- **Contact Management**: Add and manage contacts to easily connect with friends/colleagues
- **User Discovery**: Search for other users by email or name

### Device-to-Extension (Desktop Agent)
- **Desktop Agent Integration**: Connect to your Windows desktop with the optional desktop agent
- **Hardware Acceleration**: Leverages Windows APIs for efficient screen capture
- **Input Simulation**: Remote mouse and keyboard control via enigo

### Device-to-Device (Agent-to-Agent)
- **Direct Device Connection**: Connect two desktop agents directly without extension involvement
- **Host Mode**: Device shares its screen (screen sharing)
- **Client Mode**: Device views and controls remote screen (remote access)
- **Cross-Machine Control**: Full remote control between different Windows machines

### Architecture
- **Backend**: Rust server (Axum) with PostgreSQL and Redis
- **Desktop Agent**: Rust Windows application with WebRTC
- **Chrome Extension**: TypeScript/React with WebRTC
- **Signaling**: WebSocket-based WebRTC signaling server

## Project Structure
- `backend/`: Rust backend server (Axum).
- `desktop-agent/`: Rust Windows desktop agent.
- `chrome-extension/`: TypeScript Chrome extension (React).
- `docs/`: Technical documentation.
- `docker/`: Docker configurations.
- `scripts/`: Build & deployment scripts.
- `tests/`: Integration tests.
- `.github/`: CI/CD pipelines.

## Quick Start

### Prerequisites
- PostgreSQL database
- Redis server
- Rust (for backend and desktop agent)
- Node.js (for Chrome extension)

### Backend Setup

1. **Configure environment variables**:
```bash
cd backend
cp .env.example .env
# Edit .env with your database and Redis URLs
```

2. **Run migrations**:
```bash
cargo run
# Migrations run automatically on startup
```

3. **Backend will start on** `http://localhost:8080`

### Chrome Extension Setup

1. **Install dependencies**:
```bash
cd chrome-extension
npm install
```

2. **Build**:
```bash
npm run build
```

3. **Load in Chrome**:
   - Open `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `chrome-extension/dist` folder

### Desktop Agent Setup (Optional)

1. **Configure environment variables**:
```bash
cd desktop-agent
cp .env.example .env
# Edit .env with backend URL and user token
```

2. **Build and run**:
```bash
cargo run
```

## Usage

### Extension-to-Extension Screen Sharing

1. **Register/Login**: Create an account in the Chrome extension
2. **Add Contacts**: Use the Contacts tab to search and add users
3. **Connect**: Click "Connect" on a contact to send a session request
4. **Accept**: The other user will see a pending session request and can accept
5. **Share Screen**: Upon accepting, the host can select which screen to share (with audio)
6. **Remote Control**: The viewer can control the host's screen via mouse/keyboard

### Device-to-Extension (Desktop Agent)

1. **Install Desktop Agent**: Install and run the Orbit desktop agent on your Windows machine
2. **Register Device**: The agent will register itself with your account
3. **Connect**: Use the Chrome extension to connect to your registered device
4. **Remote Control**: Control your desktop remotely via the extension

### Device-to-Device (Agent-to-Agent)

**Via Chrome Extension UI:**
1. **Register Multiple Devices**: Install and register the Orbit desktop agent on at least 2 Windows machines
2. **Open Extension**: Go to the Chrome extension and navigate to the "Device-to-Device" tab
3. **Select Devices**: Choose a host device (to share screen) and client device (to view/control)
4. **Connect**: Click "Connect Devices" to establish the connection
5. **Remote Control**: The client device will display and control the host device's screen

**Via Desktop Agent (Direct):**
1. **Host Device**: Run agent in default host mode (ORBIT_MODE=host or not set)
2. **Client Device**: Run agent in client mode with environment variables:
   ```bash
   ORBIT_MODE=client
   ORBIT_HOST_DEVICE_ID=<host-device-id>
   ORBIT_CLIENT_DEVICE_ID=<your-device-id>
   ORBIT_USER_TOKEN=<your-user-token>
   ```
3. **Connection**: The client agent will connect to the host device automatically

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login
- `POST /api/v1/auth/refresh` - Refresh access token

### Contacts
- `GET /api/v1/contacts` - List contacts
- `POST /api/v1/contacts/add` - Add contact
- `DELETE /api/v1/contacts/{id}` - Remove contact
- `GET /api/v1/contacts/search?q=query` - Search users

### Sessions
- `POST /api/v1/sessions/create` - Create session (device or user)
- `POST /api/v1/sessions/{id}/accept` - Accept pending session
- `DELETE /api/v1/sessions/{id}` - End session
- `GET /api/v1/sessions/active` - Get active session (for desktop agent)
- `GET /api/v1/sessions/pending` - Get pending session requests

### Devices
- `GET /api/v1/devices` - List devices
- `POST /api/v1/devices/register` - Register device
- `POST /api/v1/devices/heartbeat` - Device heartbeat

## Database Schema

### Key Tables
- `users` - User accounts
- `contacts` - User-to-user relationships
- `devices` - Registered devices (desktop agents)
- `sessions` - Active sessions (device or user-to-user)
- `connections` - Connection metrics

## Security

- JWT-based authentication
- Device token authorization for desktop agents
- Session-based authorization for WebRTC signaling
- CORS configuration for extension access
- Password hashing with Argon2

## Development

### Running Backend
```bash
cd backend
cargo run
```

### Running Desktop Agent
```bash
cd desktop-agent
cargo run
```

### Developing Chrome Extension
```bash
cd chrome-extension
npm run dev
```

## License

MIT License - See LICENSE file for details
