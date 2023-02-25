use std::collections::HashMap;

use bevy::{
    asset::LoadState, ecs::query::QueryEntityError, math::Vec3A, prelude::*,
    render::primitives::Aabb,
};

#[derive(Component, Debug, Default)]
pub struct Marker;

#[derive(Component, Debug, Default)]
struct AabbParsed;

pub struct SceneToolsPlugin;

#[derive(Component, Debug, Clone)]
struct SceneNode {
    scene_center: Vec3,
    rotation: Quat,
    half_extents: Vec3,
}

// #[derive(Component, Debug, Clone)]
// pub struct SceneData {
//     pub rot: Quat,
//     pub half_extents: Vec3,
// }

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app.add_system_to_stage(CoreStage::Last, add_scene_nodes);
    }
}

fn add_scene_nodes(
    mut commands: Commands,
    scenes: Query<Entity, (With<Handle<Scene>>, Without<AabbParsed>, Without<Marker>)>,
    children: Query<&Children>,
    existing_meshes: Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    global_transforms: Query<&GlobalTransform>,
) -> () {
    for scene in scenes.iter() {
        let mut scene_nodes: HashMap<Entity, SceneNode> = HashMap::new();

        let scene_center = match global_transforms.get(scene) {
            Ok(global_transform) => global_transform.translation(),
            Err(_) => return (),
        };

        // commands.spawn(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::UVSphere {
        //         ..Default::default()
        //     })),
        //     transform: Transform {
        //         translation: scene_center,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // });

        println!("scene_center: {:?}", scene_center);

        get_all_meshes_from_children(
            scene_center,
            &mut commands,
            scene,
            &children,
            &existing_meshes,
            &mut scene_nodes,
            &global_transforms,
        );

        let data = get_max_half_extents(&scene_nodes, scene_center);
        let scene_aabb = Aabb {
            center: Vec3A::from(data.scene_center),
            half_extents: Vec3A::from(data.half_extents),
        };

        if !scene_nodes.is_empty() {
            // println!("Scene AABBs: {:#?}", scene_nodes);
            // println!("Scene AABB: {:#?}", scene_aabb);
            commands.entity(scene).insert(AabbParsed);
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(box_mesh_from_aabb(&scene_aabb))),
                    transform: Transform {
                        translation: Vec3::from(scene_aabb.center),
                        // translation: Vec3::ZERO,
                        rotation: data.rotation,
                        ..Default::default()
                    },
                    material: materials.add(StandardMaterial {
                        base_color: Color::Rgba {
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.2,
                        },
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert(Name::from("debug-cube"));
            // }
            commands.entity(scene).insert(Marker);

            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.25,
                        ..default()
                    })),
                    transform: Transform {
                        translation: Vec3::from(scene_aabb.center),
                        ..Default::default()
                    },
                    material: materials.add(StandardMaterial {
                        base_color: Color::Rgba {
                            red: 0.0,
                            green: 1.0,
                            blue: 0.0,
                            alpha: 0.8,
                        },
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert(Name::from("center-sphere"));
            commands.entity(scene).insert(Marker);
        }
    }
}

fn get_all_meshes_from_children<'a>(
    scene_center: Vec3,
    commands: &mut Commands,
    entity: Entity,
    children: &Query<&Children>,
    aabbs: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut scene_nodes: &'a mut HashMap<Entity, SceneNode>,
    global_transforms: &Query<&GlobalTransform>,
) -> &'a mut HashMap<Entity, SceneNode> {
    if let Ok(_children) = children.get(entity) {
        for child in _children {
            // println!("Child {:?}", child);
            commands.entity(*child).insert(Marker);

            match get_scene_aabb(&scene_center, *child, aabbs, global_transforms) {
                Ok(scene_aabb) => {
                    println!("{:#?}", scene_aabb);
                    scene_nodes.insert(*child, scene_aabb)
                }
                Err(_) => None,
            };

            scene_nodes = get_all_meshes_from_children(
                scene_center,
                commands,
                *child,
                &children,
                &aabbs,
                scene_nodes,
                global_transforms,
            );
        }
    }
    scene_nodes
}

fn get_scene_aabb(
    scene_center: &Vec3,
    entity: Entity,
    aabbs: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    global_transforms: &Query<&GlobalTransform>,
) -> Result<SceneNode, QueryEntityError> {
    let mesh_global_pos = global_transforms.get(entity)?.translation();
    let (_, aabb) = aabbs.get(entity)?;

    let (g_pos, g_rot, g_scale) = global_transforms
        .get(entity)?
        .to_scale_rotation_translation();

    // Cube #1
    //  global_transform:       (1.135, 0.732, -1.675)
    //  half_extents:           (1.0, 1.0, 1.0)
    //  scene_center:           (1.5, 0.0, -1.6)
    // SceneAabb:
    //  center:                 (2.134, 1.732, 0.675)

    // println!("scene_center: {:?}", scene_center);

    let offset = *scene_center - Vec3::from(aabb.half_extents) / 2.;

    let scene_half_extents =
        scene_center.clone().abs() + Vec3::from(aabb.center) + Vec3::from(aabb.half_extents);

    println!("################################################################");
    println!("g: {:?}", mesh_global_pos);
    println!("h: {:?}", Vec3::from(aabb.half_extents));
    println!("n: {:?}", scene_half_extents);
    println!("c: {:?}", scene_center);
    println!("################################################################");

    Ok(SceneNode {
        half_extents: scene_half_extents,
        scene_center: *scene_center,
        rotation: g_rot,
    })

    // Ok(Aabb {
    //     // center: Vec3A::from(*scene_center),
    //     center: Vec3A::from(scene_center.clone()),
    //     half_extents: Vec3A::from(scene_half_extents),
    // })
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

fn get_max_half_extents(scene_nodes: &HashMap<Entity, SceneNode>, pos: Vec3) -> SceneNode {
    let mut x_pos = 0.0;
    let mut y_pos = 0.0;
    let mut z_pos = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    let mut x_rotation = 0.0;
    let mut y_rotation = 0.0;
    let mut z_rotation = 0.0;

    for scene_node in scene_nodes.clone().into_values() {
        // TODO: Need to grab the position and rotation of the maximum extents.

        if x <= scene_node.half_extents.x {
            x = scene_node.half_extents.x;
            x_rotation = scene_node.rotation.x;
            x_pos = scene_node.scene_center.x;
        };

        if y <= scene_node.half_extents.y {
            y = scene_node.half_extents.y;
            y_rotation = scene_node.rotation.y;
            y_pos = scene_node.scene_center.y;
        };

        if z <= scene_node.half_extents.z {
            z = scene_node.half_extents.z;
            z_rotation = scene_node.rotation.z;
            z_pos = scene_node.scene_center.z;
        };

        // x_max_extent = x_max_extent.max(scene_node.half_extents.x);
        // y_max_extent = y_max_extent.max(scene_node.half_extents.y);
        // z_max_extent = z_max_extent.max(scene_node.half_extents.z);
    }

    let t = SceneNode {
        scene_center: Vec3::new(x_pos, y_pos, z_pos),
        rotation: Quat::from_xyzw(x_rotation, y_rotation, z_rotation, 0.0),
        half_extents: Vec3::new(x, y, z),
    };

    println!("t: {:#?}", t);
    t
}
