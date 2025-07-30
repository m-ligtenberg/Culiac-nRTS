# CLAUDE.md - Battle of Culiacán RTS Development Context

## 🎯 **Project Overview**

**Battle of Culiacán RTS** is a historical real-time strategy simulation of the October 17, 2019 event ("El Culiacanazo" / Black Thursday) where the Sinaloa Cartel successfully pressured the Mexican government to release Ovidio Guzmán López through coordinated urban resistance tactics.

**Educational Purpose**: This is an educational simulation designed to help understand complex geopolitical dynamics, asymmetric warfare, and the historical significance of this event - NOT to glorify violence.

---

## 🛠️ **Technical Stack & Architecture**

### **Core Technology**
- **Engine**: Bevy 0.12 (Rust game engine)
- **Language**: Rust (memory safe, cross-platform performance)
- **Platform**: Cross-platform desktop (Windows .exe, macOS .app, Linux binary)
- **Architecture**: Entity Component System (ECS) with monolithic main.rs (~1300 lines)

### **Current Visual System**
- **Hybrid Sprite System**: Colored diamond sprites + Text2D emoji overlays
- **Isometric View**: 45° rotated tactical battlefield perspective
- **Professional UI**: Real-time HUD with mission status, wave counters, health bars
- **Particle Effects**: Combat feedback, muzzle flashes, explosions, damage numbers

### **Audio System**
- **Console-based Audio**: Rich atmospheric descriptions via terminal output
- **Procedural Sound Design**: Faction-specific audio cues and radio chatter
- **Ready for Enhancement**: Prepared for actual .ogg sound file integration

---

## 📁 **Repository Structure**

```
toob-game/
├── src/
│   └── main.rs              # Monolithic game code (~1300 lines, fully functional)
├── docs/                    # All project documentation
│   ├── VISUAL_FIX.md       # Hybrid sprite system implementation
│   ├── GRAPHICS_UPGRADE.md  # UI and visual effects
│   ├── AUDIO_SYSTEM.md     # Console-based audio implementation
│   ├── GAMEPLAY.md         # Controls and mechanics
│   └── PROJECT_COMPLETION.md # Current completion status
├── assets/                  # Organized for future sprite/audio files
│   ├── audio/, sounds/, sprites/, ui/, maps/, data/
├── Cargo.toml              # Clean dependencies (Bevy + minimal)
└── README.md               # Professional project overview
```

---

## 🎮 **Current Gameplay Features**

### **Player Controls**
- **SPACE**: Deploy roadblock (cartel defensive tactic)
- **R**: Call reinforcements (cartel backup)
- **ESC**: End simulation (shows historical outcome)

### **Game Mechanics**
- **Wave-based Combat**: Progressive military assaults (5+ waves)
- **Faction System**: Cartel (player-controlled) vs Military (AI)
- **Health System**: Dynamic health bars with color coding
- **Particle Effects**: Visual combat feedback
- **Real-time UI**: Mission timer, unit counts, phase indicators

### **Unit Types**
| Unit | Emoji | Color | Faction | Role |
|------|-------|-------|---------|------|
| **Sicario** | 🔫 | Red | Cartel | Basic gunman |
| **Enforcer** | ⚔️ | Dark Red | Cartel | Heavy fighter |
| **Ovidio** | 👑 | Gold | Cartel | High Value Target |
| **Soldier** | 🪖 | Green | Military | Infantry |
| **Special Forces** | 🎯 | Bright Green | Military | Elite unit |
| **Vehicle** | 🚗 | Dark Green | Military | Transport |
| **Roadblock** | 🚧 | Orange | Cartel | Obstacle |

---

## ✅ **Current Implementation Status**

### **Completed Features**
- ✅ **Core RTS Engine**: Fully functional real-time strategy gameplay
- ✅ **Visual System**: Hybrid sprites with isometric view
- ✅ **Combat System**: Health bars, damage calculation, particle effects  
- ✅ **Wave System**: Progressive military assault waves
- ✅ **UI System**: Professional HUD with real-time updates
- ✅ **Audio System**: Console-based atmospheric audio
- ✅ **Cross-platform Build**: Native desktop executables
- ✅ **Clean Codebase**: Organized, documented, minimal warnings

### **Technical Achievements**
- ✅ **Performance**: 60+ FPS on integrated graphics
- ✅ **Memory Safety**: Rust language guarantees
- ✅ **ECS Architecture**: Scalable entity component system
- ✅ **Build System**: Optimized release builds with LTO

---

## 🚀 **Potential Development Directions**

### **Phase 5A: Asset Integration**
- Replace emoji sprites with custom pixel art
- Implement tilemap system for Culiacán city streets
- Add proper sprite animations for units and combat
- **Priority**: Medium (current emoji system works well)

### **Phase 5B: Advanced Audio**
- Replace console audio with actual .ogg sound effects
- Implement spatial audio positioning
- Add background music tracks for different phases
- Voice acting with historical quotes
- **Priority**: High (would significantly enhance immersion)

### **Phase 5C: Enhanced Gameplay**
- Camera controls (pan, zoom, follow units)
- Unit selection and direct command system
- Multiple mission phases beyond single battle
- AI improvements for military tactics
- **Priority**: High (core gameplay enhancement)

### **Phase 5D: Campaign Structure**
- Multiple missions covering the entire October 17, 2019 timeline
- Different neighborhoods of Culiacán as separate maps
- Progressive difficulty and historical accuracy
- Political pressure mechanics affecting government decisions
- **Priority**: Medium (expansion content)

### **Phase 5E: Technical Enhancements**
- Modularize the monolithic main.rs into separate files
- Save/load system for campaign progress
- Configuration file for gameplay settings
- Performance profiling and optimization
- **Priority**: Medium (code quality improvements)

---

## 🎯 **Key Design Principles**

### **Historical Accuracy**
- Based on documented events of October 17, 2019
- Educational focus on geopolitical dynamics
- Avoids glorification of violence
- Presents complex moral and political situations objectively

### **Technical Excellence**
- Memory-safe Rust implementation
- Cross-platform compatibility
- Performance-optimized ECS architecture
- Clean, maintainable code structure

### **Player Experience**
- Intuitive controls and clear visual feedback
- Educational value through interactive simulation
- Engaging tactical gameplay
- Professional presentation quality

---

## 🔧 **Development Workflow**

### **Building & Testing**
```bash
# Development build
cargo run

# Release build  
cargo build --release

# Cross-platform builds ready for distribution
```

### **Code Organization**
- **Main Systems**: Combat, Wave spawning, UI updates, Input handling
- **Components**: Position, Health, Faction, Combat stats
- **Resources**: GameState, WaveTimer, Score tracking
- **ECS Architecture**: Clean separation of data and logic

### **Performance Considerations**
- Bevy ECS handles entity management efficiently
- Particle systems use automatic cleanup
- UI updates only when necessary
- Memory-safe Rust prevents common game engine issues

---

## 📚 **Important Context for Development**

### **Educational Mission**
This game serves as an educational tool to understand:
- Asymmetric warfare dynamics in urban environments
- Political pressure and crisis decision-making
- Complex relationships between organized crime and state authority
- Historical significance of the "El Culiacanazo" event

### **Technical Constraints**
- Must remain cross-platform desktop application
- Should maintain educational focus over graphic violence
- Performance must remain accessible (integrated graphics support)
- Code should remain maintainable and well-documented

### **Success Metrics**
- **Educational Value**: Players understand historical context
- **Technical Quality**: Stable, performant, cross-platform
- **Gameplay Engagement**: Compelling tactical decision-making
- **Historical Accuracy**: Faithful to documented events

---

## 🎯 **Immediate Development Opportunities**

### **High Impact, Low Effort**
1. **Audio File Integration**: Replace console audio with actual sound effects
2. **Unit Selection**: Click-to-select units for direct control
3. **Camera Controls**: Pan and zoom for better battlefield view
4. **Save System**: Basic save/load for campaign progress

### **Medium Impact, Medium Effort**  
1. **Code Modularization**: Split main.rs into logical modules
2. **Advanced AI**: Smarter military unit tactics
3. **Multiple Maps**: Different Culiacán neighborhoods
4. **Animation System**: Unit movement and combat animations

### **High Impact, High Effort**
1. **Campaign Mode**: Multi-mission structure covering entire event
2. **3D Graphics**: Upgrade from 2D isometric to full 3D
3. **Multiplayer**: Historical scenario with multiple players
4. **VR Support**: Immersive battlefield experience

---

**Current Status**: Fully functional, professional-quality RTS ready for enhancement and expansion. The foundation is solid - focus on features that enhance the educational and gameplay experience while maintaining historical accuracy and technical excellence.