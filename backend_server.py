"""
Simple backend server for testing Sentinel reverse proxy.

This is a basic HTTP server that runs on a configurable port and responds
to all requests with a simple JSON response indicating which backend served it.
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys
import time

class BackendHandler(BaseHTTPRequestHandler):
    """Simple HTTP request handler for backend server."""
    
    def do_GET(self):
        """Handle GET requests."""
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        
        response = {
            'backend': self.server.backend_name,
            'port': self.server.server_port,
            'path': self.path,
            'method': 'GET',
            'timestamp': time.time()
        }
        
        self.wfile.write(json.dumps(response).encode())
    
    def do_POST(self):
        """Handle POST requests."""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)
        
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        
        response = {
            'backend': self.server.backend_name,
            'port': self.server.server_port,
            'path': self.path,
            'method': 'POST',
            'body_length': content_length,
            'timestamp': time.time()
        }
        
        self.wfile.write(json.dumps(response).encode())
    
    def log_message(self, format, *args):
        """Custom log format."""
        sys.stdout.write(f"[{self.server.backend_name}] {format % args}\n")

def run_backend(port, name):
    """Run a backend server on the specified port."""
    server = HTTPServer(('localhost', port), BackendHandler)
    server.backend_name = name
    
    print(f"Starting backend server '{name}' on port {port}")
    print(f"Backend URL: http://localhost:{port}")
    print("Press Ctrl+C to stop")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print(f"\nShutting down backend '{name}'")
        server.shutdown()

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python backend_server.py <port> [name]")
        print("Example: python backend_server.py 3000 backend-1")
        sys.exit(1)
    
    port = int(sys.argv[1])
    name = sys.argv[2] if len(sys.argv) > 2 else f"backend-{port}"
    
    run_backend(port, name)
