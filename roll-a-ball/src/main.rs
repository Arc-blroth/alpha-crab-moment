#![doc = include_str!("../../README.md")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]

#[cfg(all(not(debug_assertions), feature = "bevy_dyn"))]
compile_error!("Bevy should not be dynamically linked for release builds!");

use std::f32::consts::FRAC_PI_4;

use bevy::math::vec3;
use bevy::prelude::shape::{Icosphere, Plane};
use bevy::prelude::*;
use bevy::window::WindowDescriptor;
use bevy::{log, DefaultPlugins};
use bevy_dolly::prelude::*;
use bevy_dolly::system::DollyComponent;
use bevy_rapier3d::prelude::*;

pub mod score;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct FakeBall;

#[derive(Component)]
struct MainCamera;

fn main() {
    log::info!("hello acm.studio!");
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Roll A Ball!".to_string(),
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb_u8(119, 182, 254)))
        .add_dolly_component(MainCamera)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup)
        .add_startup_system(score::setup)
        .add_system(input)
        .add_system(update_camera)
        .add_system(respawn)
        .add_system(score::update_score)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera);

    commands
        .spawn()
        .insert(
            Rig::builder()
                .with(MovableLookAt::from_position_target(Vec3::ZERO))
                .build(),
        )
        .insert(MainCamera);

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 80_000.0,
            shadows_enabled: true,
            shadow_projection: OrthographicProjection {
                left: -10.0,
                right: 10.0,
                bottom: -10.0,
                top: 10.0,
                near: -10.0,
                far: 10.0,
                ..default()
            },
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_x(-FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Plane { size: 10.0 }.into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb_u8(33, 99, 51),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(5.0, 0.05, 5.0));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(
                Icosphere {
                    radius: 0.5,
                    subdivisions: 20,
                }
                .into(),
            ),
            material: materials.add(StandardMaterial {
                // paramters from the filament material guide
                base_color: Color::rgb(1.00, 0.85, 0.57),
                metallic: 0.97,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(1.2))
        .insert(ExternalForce::default())
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 2.0,
        })
        .insert(Ball);

    commands
        .spawn_bundle(TransformBundle::default())
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(1.2))
        .insert(ExternalForce::default())
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 2.0,
        })
        .insert(LockedAxes::TRANSLATION_LOCKED)
        .insert(CollisionGroups::new(Group::empty(), Group::empty()))
        .insert(FakeBall);
}

fn get_yaw(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::YXZ).0
}

fn input(
    keys: Res<Input<KeyCode>>,
    mut ball: Query<&mut ExternalForce, (With<Ball>, Without<FakeBall>)>,
    mut fake_ball: Query<(&mut ExternalForce, &Transform), (With<FakeBall>, Without<Ball>)>,
) {
    let mut ball_force = ball.single_mut();
    let (mut fake_ball_force, fake_ball_transform) = fake_ball.single_mut();
    let yaw = get_yaw(fake_ball_transform);

    ball_force.force = vec3(0.0, 0.0, 0.0);
    ball_force.torque = vec3(0.0, 0.0, 0.0);
    fake_ball_force.torque = vec3(0.0, 0.0, 0.0);
    if keys.any_pressed([KeyCode::W, KeyCode::Up]) {
        ball_force.force = vec3(4.0 * yaw.sin(), 0.0, 4.0 * yaw.cos());
    }
    if keys.any_pressed([KeyCode::S, KeyCode::Down]) {
        ball_force.force = vec3(4.0 * -yaw.sin(), 0.0, 4.0 * -yaw.cos());
    }
    if keys.any_pressed([KeyCode::Space]) {
        ball_force.force.y = 9.81;
    }
    if keys.any_pressed([KeyCode::A, KeyCode::Left]) {
        ball_force.torque = vec3(0.0, 0.25, 0.0);
        fake_ball_force.torque = vec3(0.0, 0.25, 0.0);
    }
    if keys.any_pressed([KeyCode::D, KeyCode::Right]) {
        ball_force.torque = vec3(0.0, -0.25, 0.0);
        fake_ball_force.torque = vec3(0.0, -0.25, 0.0);
    }
}

fn update_camera(
    ball: Query<&Transform, With<Ball>>,
    fake_ball: Query<&Transform, With<FakeBall>>,
    mut camera_rig: Query<&mut Rig>,
) {
    let ball = ball.single();
    let fake_ball = fake_ball.single();
    let mut camera_rig = camera_rig.single_mut();
    let yaw = get_yaw(fake_ball);
    let rot = Quat::from_axis_angle(vec3(0.0, 1.0, 0.0), yaw);

    camera_rig
        .driver_mut::<MovableLookAt>()
        .set_position_target(ball.translation - rot * vec3(0.0, 1.0, 0.0), rot);
}

fn respawn(mut ball: Query<&mut Transform, With<Ball>>) {
    let mut ball = ball.single_mut();
    if ball.translation.y < -50.0 {
        ball.translation = vec3(0.0, 1.0, 0.0);
    }
}
