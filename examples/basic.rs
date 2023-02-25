use bevy::{asset::LoadState, prelude::*, window::PresentMode};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_scene_tools::{Marker, SceneToolsPlugin};
use display_info::DisplayInfo;
use std::default;

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
    let screen_size = get_window_size().unwrap();
    let window_pos_x = (screen_size.0 as f32 * 0.66667) as f32;
    let window_pos_y = 0.0;
    let screen_size = screen_size.0 as f32 - window_pos_x;

    App::new()
        .add_plugin(SceneToolsPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: screen_size,
                height: screen_size,
                title: GAME_TITLE.to_string(),
                resizable: false,
                present_mode: PresentMode::AutoVsync,
                position: WindowPosition::At(Vec2::new(window_pos_x, window_pos_y)),
                ..Default::default()
            },
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(setup)
        .add_system(camera_controls)
        // .add_system(window_resize_system)
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

fn get_window_size() -> Result<(i32, i32), ()> {
    let display_infos = DisplayInfo::all().unwrap();
    let primary_displays = display_infos
        .iter()
        .filter(|d| d.is_primary)
        .collect::<Vec<&DisplayInfo>>();
    if let Some(primary_display) = primary_displays.first() {
        return Ok((primary_display.width as i32, primary_display.height as i32));
    }
    return Err(());
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

    const SCENE_CENTER: Vec3 = Vec3::new(1.5, 0.0, -1.6);

    let scene: Handle<Scene> = asset_server.load("one_cube.glb#Scene0");
    // let scene: Handle<Scene> = asset_server.load("two_cubes.glb#Scene0");
    // let scene: Handle<Scene> = asset_server.load("ship.gltf#Scene0");
    let entity_id = commands
        .spawn((SceneBundle {
            scene: scene.clone(),
            transform: Transform {
                translation: SCENE_CENTER,
                scale: Vec3::splat(1.0),
                ..Default::default()
            },
            ..default()
        },))
        // .insert(Marker)
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
