use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::prelude::*;

// Components
#[derive(Component)]
struct Submarine;

#[derive(Component)]
struct Fish;

#[derive(Component)]
struct CameraFollow;

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Oxygen {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Score {
    value: u32,
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
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<GameState>()
        .init_resource::<CameraState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            submarine_movement,
            camera_follow,
            mouse_camera_control,
            fish_movement,
            oxygen_system,
            collect_fish,
            ui_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
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
    let submarine_entity = commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        Submarine,
        Health { current: 100.0, max: 100.0 },
        Oxygen { current: 100.0, max: 100.0 },
        Score { value: 0 },
        RigidBody::Dynamic,
        Collider::cylinder(0.7, 2.0),
        Velocity::zero(),
        GravityScale(0.0),
    ))
    .id();

    // Add child entities for the submarine parts
    commands.entity(submarine_entity).with_children(|parent| {
        // Main hull (cylinder)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.7, height: 4.0, resolution: 32, segments: 1 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::IDENTITY,
            ..default()
        });
        
        // Bow (front sphere)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.7, sectors: 32, stacks: 16 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        });
        
        // Stern (back sphere)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.7, sectors: 32, stacks: 16 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.5),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            ..default()
        });
        
        // Horizontal stabilizers (wings)
        let wing_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..default()
        });
        
        // Left wing
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.8, 0.2, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(-0.9, -0.2, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
        
        // Right wing
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.8, 0.2, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(0.9, -0.2, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
        
        // Vertical stabilizer (rudder)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 0.6, 0.4))),
            material: wing_material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 0.7).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        });
    });

    // Ocean floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0, subdivisions: 0 })),
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

    // Spawn some fish
    for i in 0..10 {
        let x = (i as f32 - 5.0) * 8.0;
        let y = (i % 3) as f32 * 3.0 - 5.0;
        let z = (i % 2) as f32 * 10.0 - 5.0;
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.3, sectors: 16, stacks: 8 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.8, 0.6, 0.2),
                    ..default()
                }),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            Fish,
            RigidBody::Dynamic,
            Collider::ball(0.3),
            Velocity::zero(),
            GravityScale(0.0),
        ));
    }

    // UI
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Submarine Game\nWASD: Move\nSpace: Up\nShift: Down\nCollect fish to score points!",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn submarine_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut submarine_query: Query<(&mut Velocity, &mut Transform), With<Submarine>>,
    time: Res<Time>,
) {
    if let Ok((mut velocity, mut transform)) = submarine_query.get_single_mut() {
        let mut direction = Vec3::ZERO;
        let speed = 10.0;

        if keyboard_input.pressed(KeyCode::W) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction.x += 1.0;
        }
        // Only allow upward movement if not at the surface
        if keyboard_input.pressed(KeyCode::Space) && transform.translation.y < 0.0 {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction.y -= 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            velocity.linvel = direction * speed;
        } else {
            velocity.linvel *= 0.9; // Apply some drag
        }

        // Apply buoyancy force (upward force when underwater)
        if transform.translation.y < 0.0 {
            let buoyancy_force = 15.0; // Upward force
            velocity.linvel.y += buoyancy_force * time.delta_seconds();
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
    camera_state: Res<CameraState>,
) {
    if let Ok(submarine_transform) = submarine_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
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

fn mouse_camera_control(
    mut camera_state: ResMut<CameraState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let sensitivity = 0.005;

    for event in mouse_motion_events.read() {
        camera_state.yaw -= event.delta.x * sensitivity;
        camera_state.pitch -= event.delta.y * sensitivity;
        
        // Clamp pitch to prevent camera flipping
        camera_state.pitch = camera_state.pitch.clamp(-1.0, 1.0);
    }
}

fn fish_movement(
    mut fish_query: Query<&mut Transform, With<Fish>>,
    time: Res<Time>,
) {
    for mut fish_transform in fish_query.iter_mut() {
        let elapsed_time = time.elapsed_seconds();
        let offset = Vec3::new(
            (elapsed_time * 0.5).sin() * 2.0,
            (elapsed_time * 0.3).sin() * 1.0,
            (elapsed_time * 0.7).cos() * 2.0,
        );
        fish_transform.translation += offset * time.delta_seconds() * 0.5;
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
        game_state.oxygen -= time.delta_seconds() * 2.0;
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
            let distance = submarine_transform.translation.distance(fish_transform.translation);
            if distance < 2.0 {
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
    mut ui_query: Query<&mut Text>,
) {
    if let Ok(mut text) = ui_query.get_single_mut() {
        let (speed, depth, orientation) = if let Ok((transform, velocity)) = submarine_query.get_single() {
            let speed = velocity.linvel.length();
            let depth = -transform.translation.y; // Negative because Y is up in world space
            let orientation = transform.rotation.to_euler(EulerRot::YXZ);
            (speed, depth, orientation)
        } else {
            (0.0, 0.0, (0.0, 0.0, 0.0))
        };

        text.sections[0].value = format!(
            "Submarine Game\n\nScore: {}\nHealth: {:.1}%\nOxygen: {:.1}%\n\nSpeed: {:.1} m/s\nDepth: {:.1} m\nPitch: {:.1}°\nYaw: {:.1}°\nRoll: {:.1}°\n\nWASD: Move\nSpace: Up\nShift: Down\nCollect fish to score points!",
            game_state.score,
            game_state.health,
            game_state.oxygen,
            speed,
            depth,
            orientation.1.to_degrees(),
            orientation.0.to_degrees(),
            orientation.2.to_degrees()
        );
    }
}
