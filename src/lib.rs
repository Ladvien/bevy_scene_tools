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

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app.add_system_to_stage(CoreStage::Last, add_scene_aabbs);
    }
}

fn add_scene_aabbs(
    mut commands: Commands,
    scenes: Query<Entity, (With<Handle<Scene>>, Without<AabbParsed>, Without<Marker>)>,
    children: Query<&Children>,
    existing_meshes: Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    global_transforms: Query<&GlobalTransform>,
) -> () {
    for scene in scenes.iter() {
        let mut scene_aabbs: HashMap<Entity, Aabb> = HashMap::new();

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
            &mut scene_aabbs,
            &global_transforms,
        );

        let scene_aabb = Aabb {
            center: Vec3A::from(scene_center),
            half_extents: Vec3A::from(get_max_half_extents(&scene_aabbs, scene_center)),
        };

        if !scene_aabbs.is_empty() {
            // println!("Scene AABBs: {:#?}", scene_aabbs);
            // println!("Scene AABB: {:#?}", scene_aabb);
            commands.entity(scene).insert(AabbParsed);
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(box_mesh_from_aabb(&scene_aabb))),
                    transform: Transform {
                        translation: Vec3::from(scene_aabb.center),
                        // translation: Vec3::ZERO,
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
        }
    }
}

fn get_all_meshes_from_children<'a>(
    scene_center: Vec3,
    commands: &mut Commands,
    entity: Entity,
    children: &Query<&Children>,
    aabbs: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut scene_aabbs: &'a mut HashMap<Entity, Aabb>,
    global_transforms: &Query<&GlobalTransform>,
) -> &'a mut HashMap<Entity, Aabb> {
    if let Ok(_children) = children.get(entity) {
        for child in _children {
            // println!("Child {:?}", child);
            commands.entity(*child).insert(Marker);

            match get_scene_aabb(&scene_center, *child, aabbs, global_transforms) {
                Ok(scene_aabb) => {
                    println!("{:#?}", scene_aabb);
                    scene_aabbs.insert(*child, scene_aabb)
                }
                Err(_) => None,
            };

            scene_aabbs = get_all_meshes_from_children(
                scene_center,
                commands,
                *child,
                &children,
                &aabbs,
                scene_aabbs,
                global_transforms,
            );
        }
    }
    scene_aabbs
}

fn get_scene_aabb(
    scene_center: &Vec3,
    entity: Entity,
    aabbs: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    global_transforms: &Query<&GlobalTransform>,
) -> Result<Aabb, QueryEntityError> {
    let mesh_global_pos = global_transforms.get(entity)?.translation();
    let (_, aabb) = aabbs.get(entity)?;

    // Cube #1
    //  global_transform:       (1.135, 0.732, -1.675)
    //  half_extents:           (1.0, 1.0, 1.0)
    //  scene_center:           (1.5, 0.0, -1.6)
    // SceneAabb:
    //  center:                 (2.134, 1.732, 0.675)

    // println!("scene_center: {:?}", scene_center);
    let scene_half_extents = mesh_global_pos.abs() - scene_center.clone().abs()
        + Vec3::from(aabb.center)
        + Vec3::from(aabb.half_extents);
    println!("################################################################");
    println!("g: {:?}", mesh_global_pos);
    println!("h: {:?}", Vec3::from(aabb.half_extents));
    println!("n: {:?}", scene_half_extents);
    println!("c: {:?}", scene_center);
    println!("################################################################");

    Ok(Aabb {
        // center: Vec3A::from(*scene_center),
        center: Vec3A::from(scene_center.clone()),
        half_extents: Vec3A::from(scene_half_extents),
    })
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

fn get_max_half_extents(aabbs: &HashMap<Entity, Aabb>, pos: Vec3) -> Vec3 {
    let mut x_max_extent: f32 = 0.0;
    let mut y_max_extent: f32 = 0.0;
    let mut z_max_extent: f32 = 0.0;
    for aabb in aabbs.clone().into_values() {
        x_max_extent = x_max_extent.max(aabb.half_extents.x);
        y_max_extent = y_max_extent.max(aabb.half_extents.y);
        z_max_extent = z_max_extent.max(aabb.half_extents.z);
    }

    let t = Vec3::new(x_max_extent, y_max_extent, z_max_extent);
    println!("t: {:#}", t);
    t
}
