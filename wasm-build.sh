#!/bin/bash
set -e

echo "Building Inochi Particle Life System for WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not available. Please install Rust."
    exit 1
fi

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean
rm -rf pkg/
rm -rf www/dist/

# Add wasm32 target if not present
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Build with wasm-pack
echo "Building WebAssembly module..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --scope inochi \
    -- --features web

# Create web directory structure if it doesn't exist
mkdir -p www/dist

# Copy generated files to web directory
echo "Copying WebAssembly files..."
cp pkg/*.wasm www/dist/
cp pkg/*.js www/dist/
cp pkg/package.json www/dist/

# Generate HTML wrapper if it doesn't exist
if [ ! -f www/index.html ]; then
    echo "Generating HTML wrapper..."
    cat > www/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Inochi Particle Life System</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            background-color: #0a0a0a;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            color: white;
        }
        
        #container {
            text-align: center;
            max-width: 1400px;
            width: 100%;
            padding: 20px;
        }
        
        canvas {
            border: 1px solid #333;
            border-radius: 8px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
            max-width: 100%;
            height: auto;
        }
        
        .controls {
            margin: 20px 0;
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            gap: 10px;
        }
        
        button {
            background: linear-gradient(45deg, #667eea 0%, #764ba2 100%);
            border: none;
            color: white;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
            font-size: 14px;
            transition: transform 0.2s;
        }
        
        button:hover {
            transform: translateY(-2px);
        }
        
        button:active {
            transform: translateY(0);
        }
        
        .info {
            margin-top: 10px;
            font-size: 12px;
            opacity: 0.7;
        }
        
        .loading {
            font-size: 18px;
            margin: 20px 0;
        }
        
        .error {
            color: #ff6b6b;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div id="container">
        <h1>🎆 Inochi Particle Life System</h1>
        <div id="loading" class="loading">Loading WebAssembly module...</div>
        <div id="error" class="error" style="display: none;"></div>
        
        <div id="app-container" style="display: none;">
            <div class="controls">
                <button onclick="resetSimulation()">Reset</button>
                <button onclick="togglePause()">Pause/Play</button>
                <button onclick="changePreset('ParticleLife')">Particle Life</button>
                <button onclick="changePreset('Flocking')">Flocking</button>
                <button onclick="changePreset('Gravity')">Gravity</button>
                <button onclick="changePreset('Electromagnetic')">Electromagnetic</button>
                <button onclick="changePreset('Brownian')">Brownian Motion</button>
                <button onclick="changePreset('ReactionDiffusion')">Reaction-Diffusion</button>
            </div>
            
            <canvas id="nannou-canvas"></canvas>
            
            <div class="info">
                <p>🖱️ Click and drag to pan • 🔄 Mouse wheel to zoom • ⌨️ Press R to reset camera</p>
                <p id="particle-count">Particles: 0</p>
                <p id="fps">FPS: --</p>
            </div>
        </div>
    </div>

    <script type="module">
        import init, { 
            start_simulation, 
            reset_simulation,
            toggle_pause,
            change_preset,
            get_particle_count,
            get_fps
        } from './dist/inochi.js';

        let isInitialized = false;
        let isPaused = false;

        async function run() {
            try {
                // Initialize the WebAssembly module
                await init();
                
                // Start the simulation
                await start_simulation();
                
                isInitialized = true;
                document.getElementById('loading').style.display = 'none';
                document.getElementById('app-container').style.display = 'block';
                
                // Start the info update loop
                updateInfo();
                
            } catch (error) {
                console.error('Failed to initialize:', error);
                document.getElementById('loading').style.display = 'none';
                document.getElementById('error').style.display = 'block';
                document.getElementById('error').textContent = `Error: ${error.message}`;
            }
        }

        function updateInfo() {
            if (!isInitialized) return;
            
            try {
                const particleCount = get_particle_count();
                const fps = get_fps();
                
                document.getElementById('particle-count').textContent = `Particles: ${particleCount}`;
                document.getElementById('fps').textContent = `FPS: ${fps.toFixed(1)}`;
            } catch (error) {
                console.warn('Error updating info:', error);
            }
            
            requestAnimationFrame(updateInfo);
        }

        // Global functions for buttons
        window.resetSimulation = function() {
            if (isInitialized) {
                reset_simulation();
            }
        };

        window.togglePause = function() {
            if (isInitialized) {
                isPaused = !isPaused;
                toggle_pause();
            }
        };

        window.changePreset = function(presetName) {
            if (isInitialized) {
                change_preset(presetName);
            }
        };

        // Keyboard shortcuts
        document.addEventListener('keydown', (event) => {
            if (!isInitialized) return;
            
            switch (event.key.toLowerCase()) {
                case ' ':
                    event.preventDefault();
                    togglePause();
                    break;
                case 'r':
                    event.preventDefault();
                    resetSimulation();
                    break;
                case '1':
                    changePreset('ParticleLife');
                    break;
                case '2':
                    changePreset('Flocking');
                    break;
                case '3':
                    changePreset('Gravity');
                    break;
                case '4':
                    changePreset('Electromagnetic');
                    break;
                case '5':
                    changePreset('Brownian');
                    break;
                case '6':
                    changePreset('ReactionDiffusion');
                    break;
            }
        });

        // Start the application
        run();
    </script>
</body>
</html>
EOF
fi

# Create a simple HTTP server script
cat > www/serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os
import sys

PORT = 8000

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    if len(sys.argv) > 1:
        PORT = int(sys.argv[1])
    
    with socketserver.TCPServer(("", PORT), CORSRequestHandler) as httpd:
        print(f"Server starting at http://localhost:{PORT}")
        print("Press Ctrl+C to stop the server")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServer stopped")
EOF

chmod +x www/serve.py

echo "✅ WebAssembly build complete!"
echo ""
echo "To run the web version:"
echo "  cd www && python3 serve.py"
echo "  Then open http://localhost:8000 in your browser"
echo ""
echo "Note: Make sure your browser supports WebAssembly and SharedArrayBuffer"
echo "      Chrome/Firefox with --enable-features=SharedArrayBuffer flag might be needed"