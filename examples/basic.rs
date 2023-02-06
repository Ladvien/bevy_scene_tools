use bevy::{prelude::*, window::PresentMode};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_scene_tools::{Marker, SceneToolsPlugin};

pub const SCREEN_WIDTH: f32 = 720.0;
pub const SCREEN_HEIGHT: f32 = 640.0;
pub const GAME_TITLE: &str = "Scene Tools Plugin";
pub const START_X_POX: f32 = 1000.0;
pub const START_Y_POX: f32 = 0.0;

pub const CAMERA_ROTATION_SPEED: f32 = 2.5;
pub const CAMERA_MOVEMENT_SPEED: f32 = 25.0;

pub const CAM_ORIGIN_X: f32 = 15.0;
pub const CAM_ORIGIN_Y: f32 = 15.0;
pub const CAM_ORIGIN_Z: f32 = 15.0;

fn main() {
    App::new()
        .add_plugin(SceneToolsPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                title: GAME_TITLE.to_string(),
                resizable: false,
                present_mode: PresentMode::AutoVsync,
                position: WindowPosition::At(Vec2::new(START_X_POX, START_Y_POX)),
                ..Default::default()
            },
            ..default()
        }))
        .add_startup_system(setup)
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(bevy::window::close_on_esc)
        .add_system(camera_controls)
        .run();
}

#[derive(Component)]
struct MovedScene;

#[derive(Resource, Default)]
pub struct Game {
    mechanics: Mechanics,
}

#[derive(Default)]
pub struct Mechanics {
    pub move_cooldown: Timer,
    pub rotate_cooldown: Timer,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0., 5.0, 0.),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(7., 12., 7.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    let entity_id = commands
        .spawn((SceneBundle {
            scene: asset_server.load("two_cubes.glb#Scene0"),
            transform: Transform {
                translation: Vec3::ZERO,
                scale: Vec3::splat(1.0),
                ..Default::default()
            },
            ..default()
        },))
        .insert(Marker)
        .id();

    commands
        .entity(entity_id)
        .insert(Name::new(format!("cube-{:?}", entity_id)));
}

fn camera_controls(
    keyboard: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let mut camera = camera_query.single_mut();

    let mut forward = camera.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut left = camera.left();
    left.y = 0.0;
    left = left.normalize();

    let speed = CAMERA_MOVEMENT_SPEED;
    let rotate_speed = CAMERA_ROTATION_SPEED;

    //Leafwing
    if keyboard.pressed(KeyCode::W) {
        camera.translation += forward * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::S) {
        camera.translation -= forward * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::A) {
        camera.translation += left * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::D) {
        camera.translation -= left * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::E) {
        camera.rotate_axis(Vec3::Y, rotate_speed * time.delta_seconds())
    }
    if keyboard.pressed(KeyCode::Q) {
        camera.rotate_axis(Vec3::Y, -rotate_speed * time.delta_seconds())
    }
}
