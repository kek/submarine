# 3D Submarine Game

A 3D submarine exploration game built with Rust and Bevy game engine.

## Features

- **3D Submarine Movement**: Control your submarine in 3D space
- **Underwater Environment**: Explore the ocean depths with realistic physics
- **Fish Collection**: Collect fish to earn points and restore oxygen
- **Oxygen Management**: Manage your oxygen levels - run out and you'll lose health
- **Physics-based Gameplay**: Realistic underwater physics with buoyancy
- **Dynamic Camera**: Camera follows your submarine smoothly

## Controls

- **W/A/S/D**: Move forward/left/backward/right
- **Space**: Move up
- **Shift**: Move down
- **Mouse**: Look around (camera follows automatically)

## Game Mechanics

- **Score System**: Collect fish to earn points
- **Oxygen System**: Oxygen decreases over time, collect fish to restore it
- **Health System**: If oxygen runs out, you'll start losing health
- **Physics**: Realistic underwater movement with gravity and buoyancy

## Installation

1. Make sure you have Rust installed on your system
2. Clone this repository
3. Run the game:

```bash
cargo run
```

## Dependencies

- **Bevy**: 3D game engine
- **Bevy Rapier3D**: Physics engine for realistic underwater movement
- **Bevy PBR**: Physically-based rendering for realistic graphics

## Game Objectives

1. Explore the underwater environment
2. Collect as many fish as possible
3. Manage your oxygen levels
4. Try to achieve the highest score

## Development

This game is built using:
- **Rust**: Systems programming language
- **Bevy**: Data-driven game engine
- **ECS (Entity Component System)**: For efficient game logic
- **Rapier Physics**: For realistic physics simulation

## Future Enhancements

- More detailed submarine model
- Underwater creatures and obstacles
- Power-ups and special abilities
- Multiple levels and environments
- Sound effects and music
- Multiplayer support

## Roadmap

### Sonar System Enhancements
- **Rotating Sweep Line**: Add a rotating line that spins around the sonar circle for active scanning
- **Fish Detection Blips**: Display nearby fish as dots on the sonar display
- **Distance Rings**: Add concentric circles for range indication
- **Target Tracking**: Highlight and track specific targets with directional indicators
- **Active Scanning Effects**: Visual effects when sonar detects objects
- **Sonar Range Settings**: Adjustable detection range and sensitivity

### Gameplay Improvements
- **Advanced Submarine Controls**: Add roll, pitch, and yaw controls for more realistic submarine movement
- **Sonar System**: Implement active and passive sonar with different detection ranges
- **Underwater Currents**: Add dynamic water currents that affect submarine movement
- **Depth Pressure**: Implement pressure effects at different depths
- **Submarine Damage System**: Add collision damage and repair mechanics
- **Mission System**: Create specific objectives and missions to complete

### Visual and Audio Enhancements
- **Particle Effects**: Add bubbles, underwater particles, and environmental effects
- **Dynamic Lighting**: Implement underwater lighting with depth-based color changes
- **Sound Design**: Add submarine engine sounds, underwater ambience, and sonar pings
- **Visual Filters**: Add depth-based visual effects and underwater distortion
- **Weather Effects**: Surface weather that affects underwater visibility

### Environment Expansion
- **Coral Reefs**: Add colorful coral reef environments with diverse marine life
- **Shipwrecks**: Explore sunken ships and underwater ruins
- **Caves and Tunnels**: Add underwater cave systems to explore
- **Different Biomes**: Various underwater environments (deep sea, kelp forests, etc.)
- **Day/Night Cycle**: Dynamic lighting and fish behavior changes

### Technical Improvements
- **Save/Load System**: Save game progress and submarine upgrades
- **Performance Optimization**: Improve frame rates and reduce resource usage
- **Modding Support**: Allow custom content and modifications
- **Achievement System**: Add achievements and unlockable content
- **Statistics Tracking**: Track detailed gameplay statistics and records 