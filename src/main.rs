use bevy::{
    asset::LoadState,
    gltf::{Gltf, GltfMesh},
    prelude::*,
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{prelude::*, rapier::prelude::InteractionGroups};
use bevy_flycam::PlayerPlugin;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Loading,
    InGame,
}

const POWER_SEGMENTS: f32 = 4.0;

#[derive(Resource, Default)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct PowerIndicator;

#[derive(Resource)]
struct PowerPosition(Vec3);

fn main() {
    App::new()
        .init_resource::<AssetsLoading>()
        .insert_resource(PowerPosition(Vec3::ZERO))
        .add_state(AppState::Loading)
        .add_plugins(DefaultPlugins)
        // .add_plugin(PlayerPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(WorldInspectorPlugin)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(load_assets))
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(check_load_assets))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(click)
                .with_system(restart)
        )
        .run();
}

fn load_assets(asset_server: Res<AssetServer>, mut loading: ResMut<AssetsLoading>) {
    // let mesh1: Handle<Mesh> = asset_server.load("ball.glb#Mesh0/Primitive0");
    // loading.0.push(mesh1.clone_untyped());

    let gltf: Handle<Gltf> = asset_server.load("mini_golf.glb");
    loading.0.push(gltf.clone_untyped());
}

fn check_load_assets(
    asset_server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut app_state: ResMut<State<AppState>>,
) {
    match asset_server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => {}
        LoadState::Loaded => {
            app_state.set(AppState::InGame).unwrap();
        }
        _ => {
            info!("loading assets");
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loading: Res<AssetsLoading>,
    // mut ambient_light: ResMut<AmbientLight>,
) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, 10.0, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 10.0, 0.0),
        ..default()
    });

    // plane for click-detection
    commands
        .spawn(Transform::from_xyz(0.0, 0.49, 0.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(50.0, 0.005, 50.0))
        .insert(Sensor);

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let gltf_handle = loading.0[0].clone().typed::<Gltf>();
    let gltf = gltf_assets.get(&gltf_handle).unwrap();

    // println!("scenes {:?}", gltf.named_scenes);
    // println!("nodes {:?}", gltf.named_nodes);
    // println!("meshes {:?}", gltf.named_meshes);
    // println!("mats {:?}", gltf.named_materials);

    // TODO split the floor and walls so they can have different frictions!

    let mesh = &gltf.named_meshes["Floor"];
    for m in &gltf_meshes.get(mesh).unwrap().primitives {
        let mash = meshes.get(&m.mesh).unwrap();
        commands
            .spawn(RigidBody::Fixed)
            .insert(Restitution::new(0.2))
            .insert(Friction::new(0.2))
            .insert(Collider::from_bevy_mesh(&mash, &ComputedColliderShape::TriMesh).unwrap());
    }

    let mesh = &gltf.named_meshes["Wall"];
    for m in &gltf_meshes.get(mesh).unwrap().primitives {
        let mash = meshes.get(&m.mesh).unwrap();
        commands
            .spawn(RigidBody::Fixed)
            .insert(Restitution::new(0.5))
            .insert(Friction::new(0.0))
            .insert(Collider::from_bevy_mesh(&mash, &ComputedColliderShape::TriMesh).unwrap());
    }

    commands
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(SceneBundle {
            scene: asset_server.load("mini_golf.glb#Scene0"),
            ..default()
        });

    commands
        .spawn(SceneBundle {
            scene: asset_server.load("ball.glb#Scene0"),
            ..default()
        })
        .insert(Transform::from_xyz(-6.5, 5.0, 0.0))
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.375))
        .insert(Restitution::coefficient(0.2))
        .insert(ExternalImpulse::default())
        .insert(Friction::new(0.0))
        .insert(ColliderMassProperties::Density(3.0))
        .insert(Ccd::enabled())
        .insert(Ball);
}

fn restart(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    q_ball: Query<Entity, With<Ball>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        if let Ok(ball) = q_ball.get_single() {
            commands.entity(ball).despawn_recursive();
        }

        commands
            .spawn(SceneBundle {
                scene: asset_server.load("ball.glb#Scene0"),
                ..default()
            })
            .insert(Transform::from_xyz(-6.5, 5.0, 0.0))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.375))
            .insert(Restitution::coefficient(0.2))
            .insert(ExternalImpulse::default())
            .insert(Friction::new(0.0))
            .insert(ColliderMassProperties::Density(3.0))
            .insert(Ccd::enabled())
            .insert(Ball);
    }
}

fn click(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut q_ball: Query<(&Transform, &mut ExternalImpulse), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_power: Query<Entity, With<PowerIndicator>>,
    mut power_pos: ResMut<PowerPosition>,
) {
    let (trans, mut impulse) = q_ball.get_single_mut().unwrap();
    if buttons.pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera.get_single().unwrap();
        let window = windows.get_primary().unwrap();
        let cursor_pos_screen = window.cursor_position().unwrap();
        let view = camera_transform.compute_matrix();
        let (viewport_min, viewport_max) = camera.logical_viewport_rect().unwrap();
        let screen_size = camera.logical_target_size().unwrap();
        let viewport_size = viewport_max - viewport_min;
        let adj_cursor_pos = cursor_pos_screen - Vec2::new(viewport_min.x, screen_size.y - viewport_max.y);
        let projection = camera.projection_matrix();
        let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
        let near_ndc = projection.project_point3(Vec3::Z).z;
        let cursor_ndc = (adj_cursor_pos / viewport_size) * 2.0 - Vec2::ONE;
        let ndc_to_world: Mat4 = view * projection.inverse();
        let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
        let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
        let ray_direction = far - near;

        // println!("\n{:?}", near);

        let filter = QueryFilter
            ::exclude_dynamic();
            // .groups(CollisionGroups::new(
            //     Group::from_bits_truncate(0b0001),
            //     Group::from_bits_truncate(0b0011),
            // ));

        if let Some((_ent, toi)) = rapier_context.cast_ray(
            near,
            ray_direction,
            50.0,
            true,
            filter,
        ) {
            let hit_point = near + ray_direction * toi;
            power_pos.0 = hit_point;

            for ent in q_power.iter() {
                commands.entity(ent).despawn();
            }

            let dist = hit_point
                .distance(trans.translation)
                .clamp(0.0, 10.0);
            let angle = f32::atan2(hit_point.z - trans.translation.z, hit_point.x - trans.translation.x);
            let seg_x = (f32::cos(angle) * dist) / POWER_SEGMENTS;
            let seg_y = (f32::sin(angle) * dist) / POWER_SEGMENTS;

            for i in 1..=(POWER_SEGMENTS as usize) {
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            radius: 0.15,
                            depth: 0.0,
                            ..default()
                        })),
                        transform: Transform::from_xyz(
                            trans.translation.x + seg_x * (i as f32),
                            trans.translation.y,
                            trans.translation.z + seg_y * (i as f32),
                        ),
                        material: materials.add(StandardMaterial {
                            base_color: Color::ALICE_BLUE,
                            perceptual_roughness: 1.0,
                            ..default()
                        }),
                        ..default()
                    })
                    .insert(RigidBody::Fixed)
                    .insert(PowerIndicator);
            }
        }
    }

    else if buttons.just_released(MouseButton::Left) {
        let mut diff = trans.translation - power_pos.0;
        diff *= Vec3::new(1.0, 0.0, 1.0);
        impulse.impulse = diff;
        // impulse.torque_impulse = diff;

        for ent in q_power.iter() {
            commands.entity(ent).despawn();
        }
    }

    // println!("{:?}", impulse.torque_impulse);
}