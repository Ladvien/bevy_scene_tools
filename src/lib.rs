use std::collections::HashMap;

use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};

#[derive(Component, Debug, Default)]
pub struct Marker;

#[derive(Component, Debug, Default)]
struct AabbParsed;

pub struct SceneToolsPlugin;

#[derive(Debug, Default)]
pub struct SceneNode {
    pub position: Vec3,
    pub aabb: Option<Aabb>,
    pub mesh_handle: Option<Handle<Mesh>>,
}

impl SceneNode {
    fn from(position: Vec3) -> Self {
        Self {
            position,
            aabb: None,
            mesh_handle: None,
        }
    }
}

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app.add_system(add_scene_nodes);
    }
}

fn add_scene_nodes(
    mut commands: Commands,
    scenes: Query<Entity, (With<Handle<Scene>>, Without<AabbParsed>)>,
    children: Query<&Children>,
    existing_meshes: Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    global_transforms: Query<&GlobalTransform>,
) -> () {
    for scene in scenes.iter() {
        let mut scene_nodes: HashMap<Entity, SceneNode> = HashMap::new();

        get_all_meshes_from_children(
            &mut commands,
            scene,
            &children,
            &existing_meshes,
            &mut scene_nodes,
            &global_transforms,
        );
        println!("Scene AABBs: {:#?}", scene_nodes);

        // # TODO: For testing, replace with a proper flag to ensure
        // scene is completely loaded before parsing.
        if !scene_nodes.is_empty() {
            commands.entity(scene).insert(AabbParsed);
            for scene_node in scene_nodes.into_values() {
                let aabb = scene_node.aabb.unwrap_or(Aabb {
                    center: Vec3A::ZERO,
                    half_extents: Vec3A::ZERO,
                });

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(box_mesh_from_aabb(&aabb))),
                        transform: Transform {
                            translation: Vec3::from(aabb.center),
                            ..Default::default()
                        },
                        material: materials.add(StandardMaterial {
                            base_color: Color::Rgba {
                                red: 1.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 0.5,
                            },
                            emissive: Color::Rgba {
                                red: 1.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 0.5,
                            },
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .insert(Name::from("debug-cube"));
            }
        }
    }
}

fn get_all_meshes_from_children<'a>(
    commands: &mut Commands,
    entity: Entity,
    children: &Query<&Children>,
    meshes: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut scene_nodes: &'a mut HashMap<Entity, SceneNode>,
    global_transforms: &Query<&GlobalTransform>,
) -> &'a mut HashMap<Entity, SceneNode> {
    if let Ok(_children) = children.get(entity) {
        for child in _children {
            commands.entity(*child).insert(Marker);
            scene_nodes = get_all_meshes_from_children(
                commands,
                *child,
                &children,
                &meshes,
                scene_nodes,
                global_transforms,
            );
        }
    }
    scene_nodes
}

fn box_mesh_from_aabb(aabb: &Aabb) -> shape::Box {
    shape::Box {
        min_x: aabb.center.x - aabb.half_extents.x,
        max_x: aabb.center.x + aabb.half_extents.x,
        min_y: aabb.center.y - aabb.half_extents.y,
        max_y: aabb.center.y + aabb.half_extents.y,
        min_z: aabb.center.z - aabb.half_extents.z,
        max_z: aabb.center.z + aabb.half_extents.z,
    }
}
