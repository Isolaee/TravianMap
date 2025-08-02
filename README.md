# Travian Map Explorer

A full-stack web application built with Rust (Axum) backend and React TypeScript frontend for exploring and managing Travian game map data.

## 🏗️ Architecture

- **Backend**: Rust server using Axum framework
- **Frontend**: React TypeScript application built with Vite
- **Communication**: RESTful API with JSON responses

## 📁 Project Structure

```
TravianMap/
├── server/          # Rust backend application
├── server/          # Rust backend application
│   ├── src/
│   │   ├── main.rs     # Main server application
│   │   └── database.rs # PostgreSQL database operations
│   └── Cargo.toml   # Rust dependencies
├── client/          # React TypeScript frontend
│   ├── src/
│   │   ├── App.tsx  # Main React component
│   │   └── App.css  # Application styles
│   └── package.json # Node.js dependencies
├── docker-compose.yml # PostgreSQL database setup
├── init.sql         # Database initialization
├── setup-database.ps1 # Database setup script
└── README.md        # This file
```

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [npm](https://www.npmjs.com/) or [yarn](https://yarnpkg.com/)
- [Docker Desktop](https://docs.docker.com/desktop/install/windows-install/) (for PostgreSQL database)

### Installation & Running

#### 1. Set Up PostgreSQL Database

```bash
# Run the database setup script
./setup-database.ps1

# Or manually with Docker Compose
docker-compose up -d
```

#### 2. Start the Rust Backend Server

```bash
cd server
cargo run
```

The server will start on `http://127.0.0.1:3001`

#### 3. Start the React Frontend

```bash
cd client
npm install
npm run dev
```

The frontend will start on `http://127.0.0.1:5173`

## �️ Database Information

### PostgreSQL Setup
- **Database**: `travian_map`
- **Host**: `localhost:5432`
- **Username**: `postgres`
- **Password**: `password`
- **Admin Interface**: http://localhost:8080 (Adminer)

### Database Features
- Automatic table creation and sample data insertion
- Indexed queries for optimal performance
- CRUD operations for villages
- Coordinate-based filtering with radius support

## �🔌 API Endpoints

### Health Check
- `GET /` - Root endpoint with server status
- `GET /health` - Server health check

### Villages & Map Data
- `GET /api/villages` - Get all villages
- `POST /api/villages` - Create a new village
- `PUT /api/villages/:id` - Update village population
- `DELETE /api/villages/:id` - Delete a village
- `GET /api/map` - Get map data (supports x,y,radius query parameters)
- `GET /api/map?x=0&y=0&radius=10` - Get villages near coordinates

### Request/Response Examples

**Create Village:**
```json
POST /api/villages
{
  "name": "New Settlement",
  "x": 10,
  "y": 15,
  "population": 500
}
```

**Update Population:**
```json
PUT /api/villages/1
{
  "population": 1200
}
```

### Response Format

```typescript
interface Village {
  id: number;
  name: string;
  x: number;
  y: number;
  population: number;
}

interface HealthResponse {
  status: string;
  message: string;
}
```

## 🛠️ Development

### Backend Development (Rust)

The server uses:
- **Axum** - Modern async web framework
- **Tokio** - Async runtime
- **Serde** - JSON serialization
- **Tower-HTTP** - CORS middleware

Key files:
- `server/src/main.rs` - Main application with routes and handlers
- `server/Cargo.toml` - Dependencies and project configuration

### Frontend Development (React + TypeScript)

The client uses:
- **React 18** - UI library
- **TypeScript** - Type safety
- **Vite** - Fast build tool and dev server
- **Modern CSS** - Responsive design

Key files:
- `client/src/App.tsx` - Main application component
- `client/src/App.css` - Application styles
- `client/package.json` - Dependencies and scripts

## 🎯 Features

- **Real-time Data**: Fetch village and map data from Rust backend
- **Interactive UI**: Filter villages by coordinates
- **Responsive Design**: Works on desktop and mobile devices
- **Error Handling**: Graceful error states and loading indicators
- **Server Status**: Real-time server health monitoring

## 🔧 Configuration

### CORS
The server is configured with permissive CORS to allow frontend connections.

### Development Ports
- Backend: `http://127.0.0.1:3001`
- Frontend: `http://127.0.0.1:5173`

## 📝 Development Tasks

### Backend Tasks
- `cargo run` - Start the development server
- `cargo build` - Build the application
- `cargo test` - Run tests

### Frontend Tasks
- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License.

## 🎮 About Travian

Travian is a browser-based strategy game where players build and manage villages, armies, and alliances. This tool helps visualize and manage map data for strategic planning.
