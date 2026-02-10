use avian2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Walls, // Layer 0 (bit 0) - walls, obstacles, boundaries
    Player,       // Layer 1
    FriendlyProj, // Layer 2
    Enemy,        // Layer 3
    HostileProj,  // Layer 4
    Pickups,      // Layer 5
}

/*
* Layer        | Collides with
* --
* Walls        | Player, FriendlyProj, Enemy, HostileProj
* Player       | Walls, Enemy, HostileProj, Pickups
* FriendlyProj | Walls, Enemy, HostileProj(unsure?)
* Enemy        | Walls, Player, FriendlyProj
* HostileProj  | Walls, Player
* Pickups      | Player

Components to add in each type of entities

WALL: CollisionLayers = CollisionLayers::new(
    GameLayer::Walls,
    [
        GameLayer::Player,
        GameLayer::FriendlyProj,
        GameLayer::Enemy,
        GameLayer::HostileProj,
    ],
)

PLAYER: CollisionLayers = CollisionLayers::new(
    GameLayer::Player,
    [
        GameLayer::Walls,
        GameLayer::Enemy,
        GameLayer::HostileProj,
        GameLayer::Pickups,
    ]
)

FRIENDLY_PROJECTILE: CollisionLayers = CollisionLayers::new(
    GameLayer::FriendlyProj,
    [GameLayer::Walls, GameLayer::Enemy]
)

ENEMY: CollisionLayers = CollisionLayers::new(
    GameLayer::Enemy,
    [
        GameLayer::Walls,
        GameLayer::Player,
        GameLayer::FriendlyProj,
    ]
)

HOSTILE_PROJECTILE: CollisionLayers = CollisionLayers::new(
    GameLayer::HostileProj,
    [GameLayer::Walls, GameLayer::Player]
)

PICKUP: CollisionLayers = CollisionLayers::new(
    GameLayer::Pickups,
    [GameLayer::Player]
)
*/
