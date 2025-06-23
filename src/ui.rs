pub fn ui_system(
    game_state: Res<GameState>,
    submarine_query: Query<(&Transform, &Velocity), With<Submarine>>,
    fish_query: Query<&Transform, With<Fish>>,
    sonar_state: Res<SonarState>,
    mut ui_query: Query<&mut Text>,
    sonar_detections: Res<SonarDetections>,
    ballast_state: Res<BallastState>,
) {
    if let Ok(mut text) = ui_query.get_single_mut() {
        let (submarine_transform, submarine_velocity);
        if let Ok((t, v)) = submarine_query.get_single() {
            submarine_transform = t;
            submarine_velocity = v;
        } else {
            static DEFAULT_TRANSFORM: Transform = Transform::IDENTITY;
            static DEFAULT_VELOCITY: Velocity = Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            };
            submarine_transform = &DEFAULT_TRANSFORM;
            submarine_velocity = &DEFAULT_VELOCITY;
        }

        let speed = submarine_velocity.linvel.length();
        let depth = -submarine_transform.translation.y; // Negative because Y is up
        let (pitch, yaw, roll) = submarine_transform.rotation.to_euler(EulerRot::YXZ);

        let fish_count = fish_query.iter().count();

        let ballast_percent = ballast_state.fill_level * 100.0;
        let air_percent = ballast_state.compressed_air * 100.0;
        let electricity_percent = ballast_state.electricity;

        let vents_status = if ballast_state.vents_open { "OPEN" } else { "CLOSED" };
        let air_valve_status = if ballast_state.air_valve_open { "OPEN" } else { "CLOSED" };
        let compressor_status = if ballast_state.compressor_on { "ON" } else { "OFF" };

        let sub_yaw_deg = yaw.to_degrees();
        let sweep_deg = sonar_state.sweep_angle.to_degrees();

        let fish_info = if !sonar_detections.fish_positions.is_empty() {
            let (_, _, fish_angle) = sonar_detections.fish_positions[0];
            format!("Fish detected at {:.1}°", fish_angle.to_degrees())
        } else {
            "No fish detected".to_string()
        };

        text.sections[0].value = format!(
            "Submarine Game\nVents: {}   Air Valve: {}   Compressor: {}\n-----------------------------\n\nScore: {}\nHealth: {:.1}%\nOxygen: {:.1}%\nBallast: {:.1}%\nCompressed Air: {:.1}%\nElectricity: {:.1}%\n\nSpeed: {:.1} m/s\nDepth: {:.1} m\nPitch: {:.1}°\nYaw: {:.1}°\nRoll: {:.1}°\n\nSonar Debug:\nSub Yaw: {:.1}°\nSweep: {:.1}°\n{}\n\nWASD: Move\nQ: Toggle Vents\nE: Toggle Air Valve\nR: Toggle Compressor\nArrow Keys: Camera\nCollect fish to score points!\nFish remaining: {}",
            vents_status,
            air_valve_status,
            compressor_status,
            game_state.score,
            game_state.health,
            game_state.oxygen,
            ballast_percent,
            air_percent,
            electricity_percent,
            speed,
            depth,
            pitch.to_degrees(),
            sub_yaw_deg,
            roll.to_degrees(),
            sub_yaw_deg,
            sweep_deg,
            fish_info,
            fish_count
        );
    }
} 