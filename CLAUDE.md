# CLAUDE.md - Battle of Culiacán RTS Development Context

[... existing content remains unchanged ...]

## 💡 **Development Memories & Best Practices**
- Only use cargo check to save computing power
- **Modular Architecture**: Breaking down monolithic code into focused modules greatly improved maintainability
- **Environmental Systems**: Weather and lighting effects significantly enhance immersion without performance cost
- **Environmental Integration**: Weather modifiers properly integrated into movement and combat systems create meaningful tactical decisions
- **Visual Feedback**: Particle effects for rain and fog provide immediate visual understanding of environmental conditions
- **Console Feedback**: Real-time environmental status updates help players understand tactical implications
- **Political Pressure System**: Historical accuracy combined with engaging gameplay mechanics
- **Save System Design**: Multiple slots with metadata provide professional user experience
- **Configuration System**: JSON persistence with validation prevents configuration corruption
- **Asset Pipeline**: Organized structure makes adding new content straightforward
- **Spatial Audio**: 3D positioned audio creates immersive battlefield atmosphere
- **Git Hooks**: Automated pre-commit quality checks prevent compilation errors and maintain code consistency
- **Development Workflow**: Quality gates with cargo fmt, check, and clippy ensure professional code standards
- Try to let the basic tasks be done by Haiku, since its a waste of computing power
- Dont mention yourself in documentation or any other paper