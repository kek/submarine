#!/bin/bash

# Submarine Game - Screenshot Capture Script
# This script captures a screenshot of the running game and updates the README

set -e

echo "🚢 Submarine Game Screenshot Capture"
echo "===================================="

# Check if required tools are available
if ! command -v screencapture &> /dev/null && ! command -v gnome-screenshot &> /dev/null && ! command -v scrot &> /dev/null; then
    echo "❌ Error: No screenshot tool found."
    echo "Please install one of: screencapture (macOS), gnome-screenshot (Linux), or scrot (Linux)"
    exit 1
fi

# Check if ImageMagick is available for optimization
OPTIMIZE_IMAGES=false
if command -v magick &> /dev/null; then
    OPTIMIZE_IMAGES=true
    echo "✨ ImageMagick found - will optimize screenshots"
elif command -v convert &> /dev/null; then
    OPTIMIZE_IMAGES=true
    echo "✨ ImageMagick (legacy) found - will optimize screenshots"
else
    echo "⚠️  ImageMagick not found - screenshots will not be optimized"
    echo "   Install ImageMagick for smaller file sizes: brew install imagemagick"
fi

# Create screenshots directory if it doesn't exist
mkdir -p screenshots

echo "🎮 Starting the submarine game..."
# Start the game in the background
cargo run --release &
GAME_PID=$!

# Wait for the game to start up
echo "⏳ Waiting for game to load (10 seconds)..."
sleep 10

# Determine which screenshot tool to use
if command -v screencapture &> /dev/null; then
    # macOS
    echo "📸 Taking screenshot with screencapture (macOS)..."
    echo "Please focus the game window and press ENTER when ready..."
    read -p ""
    screencapture -w screenshots/submarine_game.png
elif command -v gnome-screenshot &> /dev/null; then
    # Linux with GNOME
    echo "📸 Taking screenshot with gnome-screenshot..."
    echo "Please focus the game window and press ENTER when ready..."
    read -p ""
    gnome-screenshot -w -f screenshots/submarine_game.png
elif command -v scrot &> /dev/null; then
    # Linux with scrot
    echo "📸 Taking screenshot with scrot..."
    echo "Click on the game window to capture it..."
    scrot -s screenshots/submarine_game.png
fi

# Stop the game
echo "🛑 Stopping the game..."
kill $GAME_PID 2>/dev/null || true

# Check if screenshot was created
if [ -f "screenshots/submarine_game.png" ]; then
    echo "✅ Screenshot captured: screenshots/submarine_game.png"

    # Optimize the screenshot if ImageMagick is available
    if [ "$OPTIMIZE_IMAGES" = true ]; then
        echo "🔧 Optimizing screenshot..."
        original_size=$(du -h screenshots/submarine_game.png | cut -f1)

        # Use appropriate command based on available version
        if command -v magick &> /dev/null; then
            magick screenshots/submarine_game.png -resize 1400x866 -quality 90 -strip screenshots/submarine_game_optimized.png
        else
            convert screenshots/submarine_game.png -resize 1400x866 -quality 90 -strip screenshots/submarine_game_optimized.png
        fi

        # Replace original with optimized version
        mv screenshots/submarine_game_optimized.png screenshots/submarine_game.png

        optimized_size=$(du -h screenshots/submarine_game.png | cut -f1)
        echo "✅ Screenshot optimized: $original_size → $optimized_size"
    fi

    # Update README with screenshot
    echo "📝 Updating README.md..."

    # Create backup of current README
    cp README.md README.md.backup

    # Create new README with screenshot
    cat > README.md << 'EOF'
# 3D Submarine Game

![Submarine Game Screenshot](screenshots/submarine_game.png)

A 3D submarine exploration game built with Rust and Bevy game engine featuring realistic ballast tank physics and underwater bubble effects.

## ✨ Features

- **🚢 3D Submarine Physics**: Realistic submarine movement with ballast tank system
- **🫧 Bubble Effects**: Visual air bubbles when venting ballast tanks underwater
- **🌊 Ballast System**:
  - **Q** - Toggle vents (water flows in, submarine sinks, bubbles appear)
  - **E** - Toggle air valve (compressed air pushes water out, submarine rises)
  - **R** - Toggle compressor (generates compressed air at surface only)
- **🐟 Fish Collection**: Collect fish to earn points and restore oxygen
- **🫁 Oxygen Management**: Manage your oxygen levels underwater
- **📡 Sonar System**: Active sonar with rotating sweep and fish detection
- **🌊 Ocean Environment**: Realistic water surface with wave effects
- **⚡ Resource Management**: Electricity, compressed air, and ballast levels

## 🎮 Controls

### Movement
- **W/A/S/D**: Move submarine forward/left/backward/right
- **Arrow Keys**: Control camera angle

### Ballast & Systems
- **Q**: Toggle ballast vents (sink + bubbles when underwater)
- **E**: Toggle air valve (rise, uses compressed air)
- **R**: Toggle air compressor (surface only, uses electricity)

## 🌊 Game Mechanics

### Ballast Tank System
- **Empty Ballast (0%)**: Submarine is buoyant and rises
- **Full Ballast (100%)**: Submarine is heavy and sinks
- **Vents Open**: Water flows in, submarine sinks, **bubbles visible underwater**
- **Air Valve Open**: Compressed air pushes water out, submarine rises
- **No bubbles when ballast is full** - realistic physics!

### Resource Management
- **Compressed Air**: Generated by compressor at surface, consumed when blowing ballast
- **Electricity**: Powers compressor, recharges when compressor is off
- **Oxygen**: Depletes underwater, restored by collecting fish

### Realistic Physics
- **Buoyancy**: Constant upward force based on ballast level
- **Surface Operations**: Compressor only works at surface (Y ≤ 0)
- **Bubble Physics**: Bubbles only appear underwater and disappear at surface

## 🚀 Installation & Running

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable version)
- Graphics drivers with OpenGL/Vulkan support

### Quick Start
```bash
# Clone the repository
git clone <repository-url>
cd submarine

# Run the game
cargo run --release

# For development with debug info
cargo run

# Enable physics debug wireframes
cargo run -- --debug-colliders
```

## 🔧 Dependencies

- **Bevy 0.12**: Modern 3D game engine
- **Bevy Rapier3D**: Physics simulation for realistic underwater movement
- **Clap**: Command-line argument parsing
- **Rand**: Random number generation for effects

## 🎯 Gameplay Tips

1. **Surface First**: Start at surface to ensure full compressed air
2. **Dive Carefully**: Open vents (Q) to let water in and dive
3. **Watch Resources**: Monitor compressed air and electricity levels
4. **Rise Strategically**: Use air valve (E) to blow ballast and surface
5. **Collect Fish**: Swim near fish to collect them for points and oxygen
6. **Bubble Watching**: Bubbles indicate active venting - use for visual feedback

## 🛠 Development

This game demonstrates:
- **ECS Architecture**: Bevy's Entity Component System
- **3D Physics**: Rapier physics integration
- **Real-time Rendering**: PBR materials and lighting
- **Game State Management**: Resource management and system coordination
- **Particle Effects**: Dynamic bubble spawning and animation

### Project Structure
```
submarine/
├── src/
│   └── lib.rs          # Main game logic
├── assets/
│   └── fonts/          # Game fonts
├── screenshots/        # Game screenshots
└── Cargo.toml         # Dependencies
```

## 🎮 Game Systems

- **Submarine Movement**: WASD controls with realistic physics
- **Ballast Control**: Toggle vents and air valve for depth control
- **Bubble System**: Spawns bubbles when air is vented underwater
- **Fish AI**: Autonomous fish movement with collection mechanics
- **Sonar Display**: Real-time fish detection and tracking
- **Camera System**: Smooth following camera with manual control
- **Wave Simulation**: Dynamic ocean surface with realistic waves

## 🔮 Future Enhancements

- **Advanced Sonar**: Multiple detection modes and range settings
- **Mission System**: Objectives and structured gameplay
- **Submarine Upgrades**: Enhanced systems and capabilities
- **Marine Life**: More diverse underwater creatures
- **Environmental Hazards**: Obstacles and challenges
- **Multiplayer**: Cooperative or competitive gameplay
- **Sound Design**: Immersive underwater audio
- **Weather Effects**: Dynamic ocean conditions

## 📜 License

This project is built for educational and demonstration purposes using the Rust programming language and Bevy game engine.

---

**🚢 Dive into the depths and explore the underwater world! 🌊**
EOF

    echo "✅ README.md updated with screenshot!"
    echo ""
    echo "📁 Files created/updated:"
    echo "   - screenshots/submarine_game.png (screenshot)"
    echo "   - README.md (updated with screenshot)"
    echo "   - README.md.backup (backup of original)"
    echo ""
    echo "🎯 The screenshot is now ready for GitHub!"
    echo "💡 Commit both the screenshot and README to see it on GitHub:"
    echo "   git add screenshots/submarine_game.png README.md"
    echo "   git commit -m 'Add game screenshot to README'"
    echo "   git push"

else
    echo "❌ Failed to capture screenshot"
    # Stop the game if still running
    kill $GAME_PID 2>/dev/null || true
    exit 1
fi
