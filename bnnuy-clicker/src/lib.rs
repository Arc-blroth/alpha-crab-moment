#![doc = include_str!("../../README.md")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

#[cfg(all(not(debug_assertions), feature = "bevy_dyn"))]
compile_error!("Bevy should not be dynamically linked for release builds!");

use bevy::asset::AssetPlugin;
use bevy::math::{vec2, vec3};
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::camera::{DepthCalculation, ScalingMode, WindowOrigin};
use bevy::render::primitives::Aabb;
use bevy::sprite::Mesh2dHandle;
use bevy_include_assets::*;
use bevy_rapier2d::prelude::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Component, Default)]
struct Bnnuy;

#[derive(Component)]
struct TheBnnuy {
    offset: Vec2,
}

/// BnnuyFactory™, a subsidary of Pulsar Enterprises LLC
struct BnnuyFactory {
    mesh: Mesh2dHandle,
    texture: Handle<Image>,
    rainbow_color: Handle<ColorMaterial>,
}

impl BnnuyFactory {
    pub fn assemble(
        &self,
        commands: &mut Commands,
        colors: &mut ResMut<Assets<ColorMaterial>>,
        color: Option<Color>,
        location: Vec2,
    ) {
        commands
            .spawn_bundle(ColorMesh2dBundle {
                mesh: self.mesh.clone(),
                material: color.map_or_else(
                    || self.rainbow_color.clone(),
                    |x| {
                        colors.add(ColorMaterial {
                            color: x,
                            texture: Some(self.texture.clone()),
                        })
                    },
                ),
                transform: Transform::from_translation(location.extend(0.0)),
                ..default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Collider::cuboid(5.0, 5.0))
            .insert(Restitution::coefficient(2.0))
            .insert(Bnnuy);
    }
}

#[derive(Component)]
struct Ceiling;

impl Ceiling {
    fn get_transform(windows: &Res<Windows>) -> Transform {
        let window = windows.get_primary().unwrap();
        Transform::from_translation(vec3(0.0, 100.0 / window.width() * window.height() + 5.0, 0.0))
    }
}

#[derive(Default)]
struct LastCursorPos(Vec2);

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub fn start() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bnnuy Clicker 2".to_string(),
            transparent: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgba_u8(46, 178, 255, 64)))
        .init_resource::<LastCursorPos>()
        .add_plugins_with(DefaultPlugins, |group| {
            if cfg!(not(debug_assertions)) {
                group.add_before::<AssetPlugin, _>(EmbeddedAssetsPlugin::new(include_assets!(
                    "../../assets" / "bnnuy.png"
                )));
            }
            group
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup)
        .add_system(update_ceiling)
        .add_system(dup)
        .add_system(magic)
        .add_system(cleanup)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.0,
            depth_calculation: DepthCalculation::ZDifference,
            scaling_mode: ScalingMode::FixedHorizontal(100.0),
            window_origin: WindowOrigin::BottomLeft,
            ..default()
        },
        ..default()
    });

    // ground + walls
    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(Quad::new(vec2(100.0, 10.0)).into()).into(),
            material: colors.add(ColorMaterial::from(Color::rgb_u8(84, 163, 78))),
            transform: Transform::from_translation(vec3(50.0, 0.0, 0.0)),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::compound(vec![
            (vec2(0.0, 0.0), 0.0, Collider::cuboid(1000.0, 5.0)),
            (vec2(-55.0, 0.0), 0.0, Collider::cuboid(5.0, 1000.0)),
            (vec2(55.0, 0.0), 0.0, Collider::cuboid(5.0, 1000.0)),
        ]));

    let ceiling_transform = Ceiling::get_transform(&windows);
    commands
        .spawn_bundle(TransformBundle::from_transform(ceiling_transform))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(1000.0, 5.0))
        .insert(Ceiling);

    let default_bnnuy_color = Color::rgb_u8(0, 246, 255);
    let bnnuy_texture = assets.load("bnnuy.png");
    let bnnuy_factory = BnnuyFactory {
        mesh: meshes.add(Quad::new(vec2(10.0, 10.0)).into()).into(),
        texture: bnnuy_texture.clone(),
        rainbow_color: colors.add(ColorMaterial {
            color: default_bnnuy_color,
            texture: Some(bnnuy_texture),
        }),
    };
    bnnuy_factory.assemble(
        &mut commands,
        &mut colors,
        Some(default_bnnuy_color),
        ceiling_transform.translation.truncate() + vec2(50.0, -15.0),
    );

    commands.insert_resource(bnnuy_factory);
}

fn update_ceiling(mut query: Query<&mut Transform, With<Ceiling>>, windows: Res<Windows>) {
    *query.single_mut() = Ceiling::get_transform(&windows);
}

fn dup(
    mut commands: Commands,
    mut colors: ResMut<Assets<ColorMaterial>>,
    mut last_cursor_pos: ResMut<LastCursorPos>,
    bnnuy_factory: Res<BnnuyFactory>,
    mut bnnuy_query: Query<Option<(&Transform, &mut RigidBody)>, (With<Bnnuy>, Without<TheBnnuy>)>,
    mut selected_bnnuy_query: Query<
        Option<(&mut Transform, &TheBnnuy, &mut RigidBody, Entity)>,
        (With<Bnnuy>, With<TheBnnuy>),
    >,
    rapier_context: Res<RapierContext>,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(screen_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_query.single();
        let window_size = Vec2::new(window.width(), window.height());
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate().extend(0.0);

        if let Ok(Some((mut transform, the_bnnuy, mut rigid_body, entity))) = selected_bnnuy_query.get_single_mut() {
            if buttons.pressed(MouseButton::Left) {
                transform.translation = transform.rotation * -the_bnnuy.offset.extend(0.0) + world_pos;
            } else if buttons.just_released(MouseButton::Left) {
                commands.entity(entity).remove::<TheBnnuy>();
                *rigid_body = RigidBody::Dynamic;
            }
        } else if buttons.pressed(MouseButton::Left) || buttons.just_released(MouseButton::Left) {
            let aabb = Aabb::from_min_max(world_pos - Vec3::splat(0.5), world_pos + Vec3::splat(0.5));
            rapier_context.colliders_with_aabb_intersecting_aabb(aabb, |entity| {
                if let Ok(Some((transform, mut rigid_body))) = bnnuy_query.get_mut(entity) {
                    if buttons.pressed(MouseButton::Left)
                        && (world_pos.truncate() - last_cursor_pos.0).length_squared() > 0.2
                    {
                        let offset = (transform.rotation.inverse() * (world_pos - transform.translation)).truncate();
                        commands.entity(entity).insert(TheBnnuy { offset });
                        *rigid_body = RigidBody::KinematicPositionBased;
                    } else if buttons.just_released(MouseButton::Left) {
                        bnnuy_factory.assemble(
                            &mut commands,
                            &mut colors,
                            Some(Color::hsl(360.0 * rand::random::<f32>(), 1.0, 0.89)),
                            transform.translation.truncate(),
                        );
                    }
                    false
                } else {
                    true
                }
            });
        }

        last_cursor_pos.0 = world_pos.truncate();
    }
}

fn magic(mut colors: ResMut<Assets<ColorMaterial>>, time: Res<Time>, bnnuy_factory: ResMut<BnnuyFactory>) {
    if let Some(ColorMaterial { color, .. }) = colors.get_mut(&bnnuy_factory.rainbow_color) {
        let mut hsla = color.as_hsla_f32();
        hsla[0] = (hsla[0] + time.delta().as_millis() as f32 / 8.0) % 360.0;
        *color = Color::hsla(hsla[0], hsla[1], hsla[2], hsla[3]);
    }
}

fn cleanup(
    mut commands: Commands,
    mut colors: ResMut<Assets<ColorMaterial>>,
    bnnuy_query: Query<(&Transform, Entity), With<Bnnuy>>,
    bnnuy_factory: Res<BnnuyFactory>,
    windows: Res<Windows>,
) {
    let max_y = Ceiling::get_transform(&windows).translation.y - 5.0;
    let mut any = false;
    for (transform, entity) in &bnnuy_query {
        any = true;
        let translation = transform.translation;
        if translation.x < -15.0 || translation.x > 115.0 || translation.y < -15.0 || translation.y > max_y + 15.0 {
            commands.entity(entity).despawn();
        }
    }
    if !any {
        bnnuy_factory.assemble(&mut commands, &mut colors, None, vec2(50.0, max_y - 5.0));
    }
}
