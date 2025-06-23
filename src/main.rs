extern crate rand;
use bevy::{prelude::*, render::mesh::VertexAttributeValues, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;
use clap::Parser;

// Constants
const SONAR_RANGE: f32 = 20.0;
const SONAR_CENTER_X: f32 = 100.0;
const SONAR_CENTER_Y: f32 = 100.0;
const SONAR_RADIUS: f32 = 75.0;
const SWEEP_SPEED: f32 = 1.0; // radians per second
const FISH_COUNT: usize = 20;
const FISH_COLLECTION_DISTANCE: f32 = 2.0;
const BASE_BUOYANCY_FORCE: f32 = 5.0; // Constant upward buoyancy force
const BALLAST_FILL_RATE: f32 = 0.3; // Ballast fill rate per second when vents open
const BALLAST_DRAIN_RATE: f32 = 0.4; // Ballast drain rate per second when air is used
const BALLAST_BUOYANCY_FORCE: f32 = 15.0; // Buoyancy force per unit of ballast fill
const COMPRESSED_AIR_RATE: f32 = 0.2; // Compressed air generation rate per second
const COMPRESSOR_POWER_DRAIN: f32 = 0.5; // Power drain per second when compressor is on
const POWER_RECHARGE_RATE: f32 = 0.1; // Power recharge rate per second

#[derive(Parser)]
#[command(name = "submarine")]
#[command(about = "A 3D submarine game")]
struct Args {
    /// Enable physics collider wireframes
    #[arg(short, long)]
    debug_colliders: bool,
}

// Components
#[derive(Component)]
struct Submarine;

#[derive(Component)]
struct Fish;

#[derive(Component)]
struct CameraFollow;

/// Component for bubble particles
#[derive(Component)]
struct Bubble {
    timer: Timer,
}

#[derive(Component)]
struct SonarSweepLine;

#[derive(Component)]
struct SonarBlip;

#[derive(Component)]
struct WaterSurface;

#[derive(Component)]
struct FishMovement {
    direction: Vec3,
    speed: f32,
    change_direction_timer: f32,
    change_direction_interval: f32,
}

// Resources
#[derive(Resource)]
struct GameState {
    score: u32,
    health: f32,
    oxygen: f32,
}

#[derive(Resource)]
struct CameraState {
    distance: f32,
    yaw: f32,
    pitch: f32,
    target_yaw: f32, // Target yaw that follows submarine rotation
}

#[derive(Resource)]
struct SonarState {
    sweep_angle: f32,
}

#[derive(Resource)]
struct SonarDetections {
    fish_positions: Vec<(f32, f32, f32)>, // (x, y, detection_angle) positions on sonar display
}

#[derive(Resource)]
struct BallastState {
    fill_level: f32,      // 0.0 = empty (buoyant), 1.0 = full (sinks)
    vents_open: bool,     // Water flows in when open
    air_valve_open: bool, // Compressed air flows in when open
    compressed_air: f32,  // Amount of compressed air available (0.0 to 1.0)
    compressor_on: bool,  // Air compressor is running
    electricity: f32,     // Available electricity (0.0 to 100.0)
}

#[derive(Resource)]
struct WaveTime {
    elapsed: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            health: 100.0,
            oxygen: 100.0,
        }
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            distance: 15.0,
            yaw: 0.0,
            pitch: 0.0,
            target_yaw: 0.0,
        }
    }
}

impl Default for SonarState {
    fn default() -> Self {
        Self { sweep_angle: 0.0 }
    }
}

impl Default for SonarDetections {
    fn default() -> Self {
        Self {
            fish_positions: Vec::new(),
        }
    }
}

impl Default for BallastState {
    fn default() -> Self {
        Self {
            fill_level: 0.0, // Start with empty ballast tanks (buoyant)
            vents_open: false,
            air_valve_open: false,
            compressed_air: 1.0, // Start with full compressed air
            compressor_on: false,
            electricity: 100.0, // Start with full electricity
        }
    }
}

impl Default for WaveTime {
    fn default() -> Self {
        Self { elapsed: 0.0 }
    }
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .init_resource::<GameState>()
        .init_resource::<CameraState>()
        .init_resource::<SonarState>()
        .init_resource::<SonarDetections>()
        .init_resource::<BallastState>()
        .init_resource::<WaveTime>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                submarine_movement,
                ballast_control_system,
                camera_follow,
                fish_movement,
                oxygen_system,
                collect_fish,
                ui_system,
                sonar_sweep_system,
                sonar_sweep_update_system,
                sonar_detection_system,
                sonar_blip_system,
                wave_system,
                bubble_spawner_system,
                bubble_animation_system,
            ),
        );

    // Conditionally add debug render plugin based on command line argument
    if args.debug_colliders {
        app.add_plugins(RapierDebugRenderPlugin::default());
        println!("Physics collider wireframes enabled");
    } else {
        println!("Physics collider wireframes disabled (use --debug-colliders to enable)");
    }

    app.run();
}

// Helper functions
fn normalize_angle(angle: f32) -> f32 {
    (angle + 2.0 * std::f32::consts::PI) % (2.0 * std::f32::consts::PI)
}

/// Spawns bubbles near the submarine when air is vented (air_valve_open)
fn bubble_spawner_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ballast_state: Res<BallastState>,
    query: Query<&Transform, With<Submarine>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    // Only spawn bubbles if vents are open and submarine is underwater
    if ballast_state.vents_open {
        if let Ok(sub_transform) = query.get_single() {
            // Only spawn bubbles if submarine is underwater (y < 0) and ballast is not full
            if sub_transform.translation.y < 0.0 && ballast_state.fill_level < 1.0 {
                // Use a timer to control bubble spawn rate
                *timer += time.delta_seconds();
                let spawn_interval = 0.08; // seconds between bubbles
                while *timer > spawn_interval {
                    *timer -= spawn_interval;

                    // Spawn bubble at a random offset near the bottom of the sub
                    let rng = rand::random::<f32>();
                    let offset_x = (rand::random::<f32>() - 0.5) * 0.5;
                    let offset_z = (rand::random::<f32>() - 0.5) * 0.5;
                    let bubble_pos =
                        sub_transform.translation + Vec3::new(offset_x, -0.7, offset_z); // slightly below sub

                    let bubble_radius = 0.08 + rng * 0.06;
                    let bubble_color = Color::rgba(0.8, 0.9, 1.0, 0.45);

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: bubble_radius,
                                sectors: 12,
                                stacks: 8,
                            })),
                            material: materials.add(StandardMaterial {
                                base_color: bubble_color,
                                alpha_mode: AlphaMode::Blend,
                                perceptual_roughness: 0.3,
                                reflectance: 0.1,
                                ..default()
                            }),
                            transform: Transform::from_translation(bubble_pos),
                            ..default()
                        },
                        Bubble {
                            timer: Timer::from_seconds(1.0 + rng * 0.5, TimerMode::Once),
                        },
                    ));
                }
            } else {
                *timer = 0.0;
            }
        }
    } else {
        *timer = 0.0;
    }
}

/// Animates and despawns bubbles as they rise and fade out
fn bubble_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &Handle<StandardMaterial>,
        &mut Bubble,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Handle<Mesh>>,
) {
    for (entity, mut transform, material_handle, mut bubble) in query.iter_mut() {
        // Move bubble upward
        transform.translation.y += 1.7 * time.delta_seconds();

        // Despawn bubble if it reaches the water surface (y >= 0)
        if transform.translation.y >= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Fade out
        let remaining = bubble.timer.remaining_secs();
        let total = bubble.timer.duration().as_secs_f32();
        let alpha = (remaining / total).clamp(0.0, 1.0);

        // Set alpha on material
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color.set_a(alpha * 0.45);
        }

        // Shrink slightly (optional, not strictly needed for spheres)
        // If you want to shrink, uncomment below:
        // transform.scale *= 0.995;

        // Tick timer and despawn if finished
        bubble.timer.tick(time.delta());
        if bubble.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn calculate_fish_angle(local_rel: Vec3) -> f32 {
    // Calculate angle relative to submarine's forward direction
    // Forward is negative Z in submarine's local space
    // Add 90 degrees (π/2) to make forward point to the top of the sonar
    // Negate local_rel.x to fix left/right inversion
    normalize_angle((-local_rel.x).atan2(-local_rel.z) + std::f32::consts::FRAC_PI_2)
}

fn calculate_sonar_position(fish_angle: f32, distance: f32) -> (f32, f32) {
    let scaled_dist = (distance / SONAR_RANGE) * SONAR_RADIUS;
    let blip_x = SONAR_CENTER_X + scaled_dist * fish_angle.cos();
    let blip_y = SONAR_CENTER_Y - scaled_dist * fish_angle.sin(); // Negative to flip Y axis
    (blip_x, blip_y)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    // Hide mouse cursor
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor.visible = false;
    }

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow,
    ));

    // Lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Submarine (simple cylinder with rounded ends)
    let submarine_entity = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            Submarine,
            RigidBody::Dynamic,
            Collider::capsule(Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 2.0), 0.7),
            Velocity::zero(),
            GravityScale(0.0),
        ))
        .id();

    // Add child entities for the submarine parts
    commands.entity(submarine_entity).with_children(|parent| {
        // Main hull (cylinder) - now pointing along Z-axis
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.7,
                height: 4.0,
                resolution: 32,
                segments: 1,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });

        // Bow (front sphere) - at positive Z
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.7,
                sectors: 32,
                stacks: 16,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..default()
        });

        // Stern (back sphere) - at negative Z
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.7,
                sectors: 32,
                stacks: 16,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, -2.0),
            ..default()
        });

        // Horizontal stabilizers (wings) - at the stern
        let wing_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..default()
        });

        // Left wing
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.8, 0.2, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(-0.9, 0.0, -0.2),
            ..default()
        });

        // Right wing
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.8, 0.2, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(0.9, 0.0, -0.2),
            ..default()
        });

        // Vertical stabilizer (rudder) - at the stern
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 0.6, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(0.0, 0.7, -0.2),
            ..default()
        });
    });

    // Ocean floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 100.0,
                subdivisions: 0,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.1, 0.2, 0.3),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -20.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(50.0, 0.1, 50.0),
    ));

    // Water surface with realistic waves - single large surface for smooth waves
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 100.0,
                subdivisions: 100, // High subdivision for smooth wave detail
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.1, 0.3, 0.6, 0.4),
                alpha_mode: AlphaMode::Blend,
                metallic: 0.8,
                perceptual_roughness: 0.1,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        WaterSurface,
    ));

    // Spawn fish
    for i in 0..FISH_COUNT {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / FISH_COUNT as f32;
        let distance = 10.0 + (i as f32) * 2.0; // Vary distance from 10 to 48
        let x = angle.cos() * distance;
        let z = angle.sin() * distance;
        let y = -5.0 - (i as f32) * 0.5; // Vary depth

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    sectors: 16,
                    stacks: 8,
                })),
                material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            Fish,
            RigidBody::Dynamic,
            Collider::ball(0.5),
            GravityScale(0.0),
            FishMovement {
                direction: Vec3::new(0.0, 0.0, 0.0), // No movement for debugging
                speed: 0.0,
                change_direction_timer: 0.0,
                change_direction_interval: 1.0,
            },
        ));
    }

    // UI
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Left side - Main HUD
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "Submarine Game\n\nScore: 0\nHealth: 100.0%\nOxygen: 100.0%\nBallast: 0.0%\nCompressed Air: 0.0%\nElectricity: 100.0%\n\nSpeed: 0.0 m/s\nDepth: 0.0 m\nPitch: 0.0°\nYaw: 0.0°\nRoll: 0.0°\n\nSonar Debug:\nSub Yaw: 0.0°\nSweep: 0.0°\nFish Angle: 0.0°\nNo fish detected\n\nWASD: Move\nQ: Toggle Vents\nE: Toggle Air Valve\nR: Toggle Compressor\nArrow Keys: Camera\nCollect fish to score points!",
                                TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    font: asset_server.load("fonts/NotoSans-Regular.ttf"),
                                },
                            ),
                            style: Style {
                                height: Val::Auto,
                                overflow: Overflow::visible(),
                                ..default()
                            },
                            ..default()
                        },
                        UiImage::default(),
                    ));
                });

            // Right side - Sonar
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(200.0),
                        align_self: AlignSelf::FlexEnd,
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
                    ..default()
                })
                .with_children(|sonar_parent| {
                    // Sonar circle (approximated with multiple small squares)
                    for i in 0..360 {
                        let angle = i as f32 * std::f32::consts::PI / 180.0;
                        let radius = 75.0;
                        let x = 100.0 + radius * angle.cos();
                        let y = 100.0 + radius * angle.sin();

                        sonar_parent.spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(2.0),
                                height: Val::Px(2.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(x - 1.0),
                                top: Val::Px(y - 1.0),
                                ..default()
                            },
                            background_color: Color::GREEN.into(),
                            ..default()
                        });
                    }

                    // Vertical cross line
                    sonar_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(2.0),
                            height: Val::Px(150.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(99.0),
                            top: Val::Px(25.0),
                            ..default()
                        },
                        background_color: Color::GREEN.into(),
                        ..default()
                    });

                    // Horizontal cross line
                    sonar_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(2.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(25.0),
                            top: Val::Px(99.0),
                            ..default()
                        },
                        background_color: Color::GREEN.into(),
                        ..default()
                    });

                    // Center dot
                    sonar_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(6.0),
                            height: Val::Px(6.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(97.0),
                            top: Val::Px(97.0),
                            ..default()
                        },
                        background_color: Color::GREEN.into(),
                        ..default()
                    });

                    // Create blip entities for fish detection
                    for _ in 0..10 {
                        sonar_parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(6.0),
                                    height: Val::Px(6.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    ..default()
                                },
                                background_color: Color::rgba(0.0, 1.0, 0.0, 0.0).into(), // Transparent initially
                                ..default()
                            },
                            SonarBlip,
                        ));
                    }

                    // Create sweep line segments for rotating sweep effect
                    for _ in 0..20 {
                        sonar_parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(2.0),
                                    height: Val::Px(2.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(100.0),
                                    top: Val::Px(100.0),
                                    ..default()
                                },
                                background_color: Color::rgb(0.0, 1.0, 0.0).into(),
                                ..default()
                            },
                            SonarSweepLine,
                        ));
                    }
                });
        });
}

fn submarine_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut submarine_query: Query<(&mut Velocity, &mut Transform), With<Submarine>>,
    mut camera_state: ResMut<CameraState>,
    ballast_state: Res<BallastState>,
    time: Res<Time>,
) {
    if let Ok((mut velocity, mut transform)) = submarine_query.get_single_mut() {
        let mut move_direction = 0.0;
        let speed = 10.0;
        let turn_speed = 1.5; // radians/sec
        let camera_rotation_speed = 2.0; // radians/sec

        // Forward/backward in facing direction
        if keyboard_input.pressed(KeyCode::W) {
            move_direction += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_direction -= 1.0;
        }
        // Turn left/right
        if keyboard_input.pressed(KeyCode::A) {
            transform.rotate(Quat::from_rotation_y(turn_speed * time.delta_seconds()));
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.rotate(Quat::from_rotation_y(-turn_speed * time.delta_seconds()));
        }

        // Camera rotation with arrow keys
        if keyboard_input.pressed(KeyCode::Left) {
            camera_state.yaw -= camera_rotation_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Right) {
            camera_state.yaw += camera_rotation_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Up) {
            camera_state.pitch += camera_rotation_speed * time.delta_seconds();
            camera_state.pitch = camera_state.pitch.clamp(-1.0, 1.0);
        }
        if keyboard_input.pressed(KeyCode::Down) {
            camera_state.pitch -= camera_rotation_speed * time.delta_seconds();
            camera_state.pitch = camera_state.pitch.clamp(-1.0, 1.0);
        }

        // Calculate movement in local forward direction
        let mut local_velocity = Vec3::ZERO;
        if (move_direction as f32).abs() > 0.0 {
            // Forward is negative Z in standard Bevy coordinates
            local_velocity +=
                transform.rotation * Vec3::new(0.0, 0.0, -1.0) * move_direction * speed;
        }

        if local_velocity.length() > 0.0 {
            velocity.linvel = local_velocity;
        } else {
            velocity.linvel *= 0.9; // Apply some drag
        }

        // Apply realistic buoyancy force (constant upward force minus ballast weight)
        // Apply to all underwater positions, including at surface (Y <= 0)
        if transform.translation.y <= 0.0 {
            // Constant upward buoyancy force (like real physics)
            let upward_buoyancy = BASE_BUOYANCY_FORCE;

            // Downward force from ballast tanks (fills with water, making submarine heavier)
            let ballast_weight = ballast_state.fill_level * BALLAST_BUOYANCY_FORCE;

            let net_buoyancy_force = upward_buoyancy - ballast_weight;
            velocity.linvel.y += net_buoyancy_force * time.delta_seconds();
        }

        // Prevent submarine from going above the surface (Y > 0)
        if transform.translation.y > 0.0 {
            transform.translation.y = 0.0;
            // Stop upward velocity when hitting the surface
            if velocity.linvel.y > 0.0 {
                velocity.linvel.y = 0.0;
            }
        }
    }
}

fn camera_follow(
    submarine_query: Query<&Transform, With<Submarine>>,
    mut camera_query: Query<&mut Transform, (With<CameraFollow>, Without<Submarine>)>,
    mut camera_state: ResMut<CameraState>,
    time: Res<Time>,
) {
    if let Ok(submarine_transform) = submarine_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Get submarine's yaw rotation
            let submarine_yaw = submarine_transform.rotation.to_euler(EulerRot::YXZ).0;

            // Update target yaw to follow submarine rotation
            camera_state.target_yaw = submarine_yaw;

            // Smoothly interpolate camera yaw towards target yaw (rubber band effect)
            let yaw_lerp_speed = 2.0; // Adjust this for faster/slower camera following
            let angle_diff = (camera_state.target_yaw - camera_state.yaw + std::f32::consts::PI)
                % (2.0 * std::f32::consts::PI)
                - std::f32::consts::PI;
            camera_state.yaw += angle_diff * yaw_lerp_speed * time.delta_seconds();

            // Calculate camera position based on yaw and pitch
            // When yaw=0, pitch=0: camera should be behind submarine (positive Z)
            let x = camera_state.distance * camera_state.yaw.sin();
            let y = camera_state.distance * camera_state.pitch.sin() + 5.0;
            let z = camera_state.distance * camera_state.yaw.cos() * camera_state.pitch.cos();

            let target_position = submarine_transform.translation + Vec3::new(x, y, z);
            camera_transform.translation = camera_transform.translation.lerp(target_position, 0.1);
            camera_transform.look_at(submarine_transform.translation, Vec3::Y);
        }
    }
}

fn fish_movement(
    mut fish_query: Query<(&mut Transform, &mut FishMovement), With<Fish>>,
    time: Res<Time>,
) {
    for (mut fish_transform, mut fish_movement) in fish_query.iter_mut() {
        let delta_time = time.delta_seconds();

        // Update direction change timer
        fish_movement.change_direction_timer += delta_time;

        // Change direction when timer expires
        if fish_movement.change_direction_timer >= fish_movement.change_direction_interval {
            // Generate new random direction with emphasis on lateral movement
            let random_x = (fish_movement.change_direction_timer * 0.5
                + fish_transform.translation.x * 0.1)
                .sin()
                * 2.0
                - 1.0;
            let random_y = (fish_movement.change_direction_timer * 0.3
                + fish_transform.translation.y * 0.2)
                .cos()
                * 0.5
                - 0.25; // Reduced vertical movement
            let random_z = (fish_movement.change_direction_timer * 0.7
                + fish_transform.translation.z * 0.1)
                .sin()
                * 2.0
                - 1.0;

            fish_movement.direction = Vec3::new(random_x, random_y, random_z).normalize();

            // Reset timer and set new random interval (more variation)
            fish_movement.change_direction_timer = 0.0;
            fish_movement.change_direction_interval = 1.5
                + (fish_movement.change_direction_timer * 0.2
                    + fish_transform.translation.x * 0.01)
                    .sin()
                    * 2.0;
        }

        // Add some lateral swaying motion
        let sway_x =
            (fish_movement.change_direction_timer * 2.0 + fish_transform.translation.x * 0.1).sin()
                * 0.3;
        let sway_z =
            (fish_movement.change_direction_timer * 1.5 + fish_transform.translation.z * 0.1).cos()
                * 0.3;

        // Move fish in current direction with added lateral sway
        let base_movement = fish_movement.direction * fish_movement.speed * delta_time;
        let sway_movement = Vec3::new(sway_x, 0.0, sway_z) * delta_time;
        fish_transform.translation += base_movement + sway_movement;

        // Prevent fish from going above the surface (Y > 0)
        if fish_transform.translation.y > 0.0 {
            fish_transform.translation.y = 0.0;
            // Bounce off surface by inverting Y direction
            fish_movement.direction.y = -fish_movement.direction.y.abs();
        }

        // Keep fish within reasonable bounds (optional)
        let max_distance = 25.0;
        let distance_from_origin = fish_transform.translation.length();
        if distance_from_origin > max_distance {
            // Move fish back towards origin
            let direction_to_origin = -fish_transform.translation.normalize();
            fish_transform.translation += direction_to_origin * delta_time * 2.0;
        }
    }
}

fn oxygen_system(
    mut game_state: ResMut<GameState>,
    submarine_query: Query<&Transform, With<Submarine>>,
    time: Res<Time>,
) {
    let depth = if let Ok(transform) = submarine_query.get_single() {
        -transform.translation.y // Negative because Y is up in world space
    } else {
        0.0
    };

    if depth <= 0.0 {
        // At or above surface - increase oxygen
        game_state.oxygen += time.delta_seconds() * 5.0;
        game_state.oxygen = game_state.oxygen.min(100.0);
    } else {
        // Below surface - decrease oxygen
        game_state.oxygen -= time.delta_seconds() * 0.02;
        game_state.oxygen = game_state.oxygen.max(0.0);
    }

    // If oxygen runs out, health decreases
    if game_state.oxygen <= 0.0 {
        game_state.health -= time.delta_seconds() * 5.0;
        game_state.health = game_state.health.max(0.0);
    }
}

fn collect_fish(
    mut commands: Commands,
    submarine_query: Query<&Transform, With<Submarine>>,
    fish_query: Query<(Entity, &Transform), With<Fish>>,
    mut game_state: ResMut<GameState>,
) {
    if let Ok(submarine_transform) = submarine_query.get_single() {
        for (fish_entity, fish_transform) in fish_query.iter() {
            let distance = submarine_transform
                .translation
                .distance(fish_transform.translation);
            if distance < FISH_COLLECTION_DISTANCE {
                commands.entity(fish_entity).despawn();
                game_state.score += 10;
                game_state.oxygen = (game_state.oxygen + 20.0).min(100.0);
            }
        }
    }
}

fn ui_system(
    game_state: Res<GameState>,
    submarine_query: Query<(&Transform, &Velocity), With<Submarine>>,
    fish_query: Query<&Transform, With<Fish>>,
    sonar_state: Res<SonarState>,
    mut ui_query: Query<&mut Text>,
    sonar_detections: Res<SonarDetections>,
    ballast_state: Res<BallastState>,
) {
    if let Ok(mut text) = ui_query.get_single_mut() {
        let (speed, depth, orientation) =
            if let Ok((transform, velocity)) = submarine_query.get_single() {
                let speed = velocity.linvel.length();
                let depth = -transform.translation.y; // Negative because Y is up in world space
                let orientation = transform.rotation.to_euler(EulerRot::YXZ);
                (speed, depth, orientation)
            } else {
                (0.0, 0.0, (0.0, 0.0, 0.0))
            };

        let submarine_yaw = orientation.0.to_degrees();
        let sweep_angle = sonar_state.sweep_angle.to_degrees();

        // Calculate fish angle for debugging
        let fish_angle_deg =
            if let Ok((submarine_transform, _velocity)) = submarine_query.get_single() {
                if let Ok(fish_transform) = fish_query.get_single() {
                    let rel = fish_transform.translation - submarine_transform.translation;
                    // Transform to submarine's local coordinate system
                    let local_rel = submarine_transform.rotation.inverse() * rel;
                    let fish_angle = calculate_fish_angle(local_rel);
                    fish_angle.to_degrees()
                } else {
                    0.0
                }
            } else {
                0.0
            };

        // Debug fading calculations
        let fade_debug = if sonar_detections.fish_positions.len() > 0 {
            let (_, _, fish_angle) = sonar_detections.fish_positions[0];
            format!("Fish detected: {:.1}°", fish_angle.to_degrees())
        } else {
            "No fish detected".to_string()
        };

        // Create status indicators for valves and vents
        let vents_status = if ballast_state.vents_open {
            "[Vents ON]"
        } else {
            "[Vents OFF]"
        };
        let air_valve_status = if ballast_state.air_valve_open {
            "[Valve ON]"
        } else {
            "[Valve OFF]"
        };
        let compressor_status = if ballast_state.compressor_on {
            "[Compressor ON]"
        } else {
            "[Compressor OFF]"
        };

        text.sections[0].value = format!(
            "Submarine Game\n\nScore: {}\nHealth: {:.1}%\nOxygen: {:.1}%\nBallast: {:.1}% {}\nCompressed Air: {:.1}% {}\nElectricity: {:.1}% {}\n\nSpeed: {:.1} m/s\nDepth: {:.1} m\nPitch: {:.1}°\nYaw: {:.1}°\nRoll: {:.1}°\n\nSonar Debug:\nSub Yaw: {:.1}°\nSweep: {:.1}°\nFish Angle: {:.1}°\n{}\n\nWASD: Move\nQ: Toggle Vents\nE: Toggle Air Valve\nR: Toggle Compressor\nArrow Keys: Camera\nCollect fish to score points!",
            game_state.score,
            game_state.health,
            game_state.oxygen,
            ballast_state.fill_level * 100.0,
            vents_status,
            ballast_state.compressed_air * 100.0,
            air_valve_status,
            ballast_state.electricity,
            compressor_status,
            speed,
            depth,
            orientation.1.to_degrees(),
            orientation.0.to_degrees(),
            orientation.2.to_degrees(),
            submarine_yaw,
            sweep_angle,
            fish_angle_deg,
            fade_debug
        );
    }
}

fn sonar_sweep_system(mut sonar_state: ResMut<SonarState>, time: Res<Time>) {
    sonar_state.sweep_angle -= time.delta_seconds() * SWEEP_SPEED; // Counter-clockwise rotation to match angle calculations
}

fn sonar_sweep_update_system(
    sonar_state: Res<SonarState>,
    submarine_query: Query<&Transform, With<Submarine>>,
    mut sweep_line_query: Query<&mut Style, With<SonarSweepLine>>,
) {
    let num_segments = 20;

    // Get submarine's yaw rotation to make sweep relative to submarine orientation
    let submarine_yaw = if let Ok(submarine_transform) = submarine_query.get_single() {
        submarine_transform.rotation.to_euler(EulerRot::YXZ).0
    } else {
        0.0
    };

    // Position each segment along the sweep angle (clockwise)
    // Make sweep angle relative to submarine's orientation
    for (index, mut style) in sweep_line_query.iter_mut().enumerate() {
        let segment_distance = (index as f32 + 1.0) * (SONAR_RADIUS / num_segments as f32);
        let sweep_angle = sonar_state.sweep_angle + submarine_yaw;
        let segment_x = SONAR_CENTER_X + segment_distance * sweep_angle.cos();
        let segment_y = SONAR_CENTER_Y - segment_distance * sweep_angle.sin(); // Negative to flip Y axis

        style.left = Val::Px(segment_x - 1.0);
        style.top = Val::Px(segment_y - 1.0);
        style.width = Val::Px(2.0);
        style.height = Val::Px(2.0);
    }
}

fn sonar_detection_system(
    submarine_query: Query<&Transform, With<Submarine>>,
    fish_query: Query<(Entity, &Transform), With<Fish>>,
    mut sonar_detections: ResMut<SonarDetections>,
    _sonar_state: Res<SonarState>,
) {
    if let Ok(submarine_transform) = submarine_query.get_single() {
        let mut fish_positions = Vec::new();

        // Detect all fish within range
        for (_entity, fish_transform) in fish_query.iter() {
            let rel = fish_transform.translation - submarine_transform.translation;
            let dist = rel.length();
            if dist > SONAR_RANGE {
                continue;
            }

            // Transform to submarine's local coordinate system
            let local_rel = submarine_transform.rotation.inverse() * rel;

            // Calculate angle relative to submarine's forward direction
            let fish_angle = calculate_fish_angle(local_rel);

            // Convert to sonar display coordinates
            let (blip_x, blip_y) = calculate_sonar_position(fish_angle, dist);

            fish_positions.push((blip_x, blip_y, fish_angle));
        }

        sonar_detections.fish_positions = fish_positions;
    }
}

fn sonar_blip_system(
    sonar_detections: Res<SonarDetections>,
    mut blip_query: Query<(&mut Style, &mut BackgroundColor), With<SonarBlip>>,
    _sonar_state: Res<SonarState>,
) {
    for (i, (mut style, mut color)) in blip_query.iter_mut().enumerate() {
        if i < sonar_detections.fish_positions.len() {
            let (x, y, _fish_angle) = sonar_detections.fish_positions[i];
            style.left = Val::Px(x - 3.0);
            style.top = Val::Px(y - 3.0);
            *color = Color::rgb(0.0, 1.0, 0.0).into(); // Solid green
        } else {
            *color = Color::rgba(0.0, 1.0, 0.0, 0.0).into(); // Transparent
        }
    }
}

fn ballast_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut ballast_state: ResMut<BallastState>,
    submarine_query: Query<&Transform, With<Submarine>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();

    // Get submarine depth
    let depth = if let Ok(transform) = submarine_query.get_single() {
        -transform.translation.y // Negative because Y is up in world space
    } else {
        0.0
    };

    // Toggle vents (Q key) - allows water to flow into ballast tanks
    if keyboard_input.just_pressed(KeyCode::Q) {
        ballast_state.vents_open = !ballast_state.vents_open;
        // Close air valve when opening vents
        if ballast_state.vents_open {
            ballast_state.air_valve_open = false;
        }
    }

    // Toggle air valve (E key) - allows compressed air to flow into tanks
    if keyboard_input.just_pressed(KeyCode::E) {
        ballast_state.air_valve_open = !ballast_state.air_valve_open;
        // Close vents when opening air valve
        if ballast_state.air_valve_open {
            ballast_state.vents_open = false;
        }
    }

    // Toggle air compressor (R key) - generates compressed air (only at surface)
    if keyboard_input.just_pressed(KeyCode::R) {
        if depth <= 0.0 {
            ballast_state.compressor_on = !ballast_state.compressor_on;
        } else {
            // Turn off compressor if underwater
            ballast_state.compressor_on = false;
        }
    }

    // Update compressed air based on compressor (only at surface)
    if ballast_state.compressor_on && ballast_state.electricity > 0.0 && depth <= 0.0 {
        ballast_state.compressed_air += COMPRESSED_AIR_RATE * delta_time;
        ballast_state.compressed_air = ballast_state.compressed_air.min(1.0);

        // Drain electricity
        ballast_state.electricity -= COMPRESSOR_POWER_DRAIN * delta_time;
        ballast_state.electricity = ballast_state.electricity.max(0.0);
    } else if depth > 0.0 {
        // Turn off compressor if underwater
        ballast_state.compressor_on = false;
    }

    // Recharge electricity slowly when compressor is off
    if !ballast_state.compressor_on {
        ballast_state.electricity += POWER_RECHARGE_RATE * delta_time;
        ballast_state.electricity = ballast_state.electricity.min(100.0);
    }

    // Update ballast fill level based on vents and air valve
    if ballast_state.vents_open {
        // Water flows in through vents
        ballast_state.fill_level += BALLAST_FILL_RATE * delta_time;
        ballast_state.fill_level = ballast_state.fill_level.min(1.0);
    } else if ballast_state.air_valve_open && ballast_state.compressed_air > 0.0 {
        // Compressed air pushes water out
        ballast_state.fill_level -= BALLAST_DRAIN_RATE * delta_time;
        ballast_state.fill_level = ballast_state.fill_level.max(0.0);

        // Use compressed air
        ballast_state.compressed_air -= BALLAST_DRAIN_RATE * delta_time * 0.5; // Air is used slower than water
        ballast_state.compressed_air = ballast_state.compressed_air.max(0.0);

        // Turn off air valve when ballast is empty
        if ballast_state.fill_level <= 0.0 {
            ballast_state.air_valve_open = false;
        }
    }
}

fn wave_system(
    mut water_query: Query<&mut Handle<Mesh>, With<WaterSurface>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wave_time: ResMut<WaveTime>,
    time: Res<Time>,
) {
    // Update elapsed time
    wave_time.elapsed += time.delta_seconds();

    if let Ok(mesh_handle) = water_query.get_single_mut() {
        if let Some(mesh) = meshes.get_mut(&*mesh_handle) {
            // Get mesh attributes
            if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                if let VertexAttributeValues::Float32x3(positions) = positions {
                    // Create wave deformation by modifying vertex positions
                    let wave_height = 0.3;
                    let wave_speed = 1.5;
                    let target_wavelength = 0.4; // 1/10 of submarine length
                    let base_frequency = wave_speed / target_wavelength;

                    for position in positions.iter_mut() {
                        let x = position[0];
                        let z = position[2];

                        // Create wave deformation based on position
                        let time_factor = wave_time.elapsed * base_frequency;

                        // Multiple wave patterns for realistic ocean
                        let wave1 = (x * 8.0 + time_factor).sin() * wave_height * 0.4;
                        let wave2 = (z * 6.0 - time_factor * 0.7).sin() * wave_height * 0.3;
                        let wave3 = ((x + z) * 4.0 + time_factor * 1.2).sin() * wave_height * 0.2;
                        let wave4 = ((x - z) * 3.0 - time_factor * 0.5).sin() * wave_height * 0.1;

                        // Apply wave deformation to Y position
                        position[1] = wave1 + wave2 + wave3 + wave4;
                    }
                }
            }

            // Update mesh normals for proper lighting
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        }
    }
}
