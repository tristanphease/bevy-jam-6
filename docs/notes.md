
# Theme

Chain Reaction

# Plan

- ~~Tower defense game with cards? Get cards by collecting them around the map and use them to defeat enemies?~~
- ~~Tower attack game where you attack a tower and try to cause chain reactions with physics to get it to fall down?~~
- Simple 2d platformer where you swing off of chains to progress


## Todo list

- Handle player animation - done (need to improve running animation)
- Make camera follow the player - done
- Work out a way to create levels
    - Use bevy_ecs_ldtk / ltdk.io - done
    - Create basic tileset - think about a theme
- Get chains working
    - Pre-existing in world chains
    - Chains coming down - have events that trigger this? 
    - Chains from player
- Multiple levels
- Fix menu and restarting level
- Audio??

For chains:
- Add identifier to chains so they can be differentiated
- Fix collision hook so player doesn't collide with chain attached to
- make minor adjustments so it feels right