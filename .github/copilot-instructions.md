# Copilot Instructions for Travian Map Project

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

## Project Overview
This is a full-stack web application with:
- **Backend**: Rust server using Axum framework
- **Frontend**: React TypeScript application using Vite

## Architecture
- The server runs on `http://127.0.0.1:3001`
- The client runs on `http://127.0.0.1:5173` (Vite dev server)
- API endpoints are prefixed with `/api/`

## Development Guidelines

### Rust Backend (`/server`)
- Use Axum framework for HTTP handling
- Implement CORS for frontend connectivity
- Use Serde for JSON serialization/deserialization
- Follow Rust naming conventions (snake_case)
- Handle errors gracefully with proper HTTP status codes

### React Frontend (`/client`)
- Use TypeScript for type safety
- Follow React hooks patterns
- Use modern CSS with responsive design
- Handle loading states and errors gracefully
- Use fetch API for HTTP requests

### API Design
- RESTful endpoints returning JSON
- Consistent error handling
- Clear response structures
- Support for query parameters where applicable

## Code Style
- Use proper TypeScript interfaces for data structures
- Implement proper error boundaries
- Follow accessibility best practices
- Use semantic HTML elements
- Maintain clean, readable code with proper comments
