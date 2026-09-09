# OmnesAgent Web ADE

Desktop Web Application (Autonomous Agent Development Environment) for OmnesAgent.

## Features
- **Desktop-First & Responsive**: Full 3-panel ADE on desktop, adaptive 2-panel on tablet, and optimized 1-panel layout with bottom navigation bar and project drawer on mobile.
- **PTY-over-WebSocket Terminal**: Browser terminal connected directly to the gateway PTY stream.
- **Live Browser & Inspector**: Sandboxed iframe preview with developer tools and automation hooks.
- **Admin Authentication**: Argon2id password authentication, session management, and first-login password rotation.
- **PWA Ready**: Offline-capable service worker, manifest, and fast CanvasKit rendering.

## Development

```bash
# Analyze code
flutter analyze --no-pub

# Run locally in Chrome
flutter run -d chrome

# Build production bundle
flutter build web --release --web-renderer canvaskit
```
