use bevy::{prelude::*, render::primitives::Aabb};

#[derive(Component, Debug, Default)]
pub struct Marker;

pub struct SceneToolsPlugin;

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app.add_system(add_scene_aabbs);
    }
}

fn add_scene_aabbs(
    mut commands: Commands,
    scenes: Query<Entity, With<Handle<Scene>>>,
    children: Query<&Children>,
    meshes: Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
) -> () {
    for scene in scenes.iter() {
        get_all_meshes_from_children(&mut commands, scene, &children, &meshes)
    }
}

fn get_all_meshes_from_children(
    commands: &mut Commands,
    entity: Entity,
    children: &Query<&Children>,
    meshes: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
) {
    if let Ok(_children) = children.get(entity) {
        // println!("Children: {:?}", _children);
        for child in _children {
            // println!("Child ID: {:?}", child);
            if let Ok((mesh, aabb)) = meshes.get(*child) {
                println!("Mesh ID: {:?}, AABB: {:?}", mesh, aabb);
                // commands.entity(mesh).insert(Marker);
            }
            commands.entity(*child).insert(Marker);
            get_all_meshes_from_children(commands, *child, &children, &meshes)
        }
    }
}
