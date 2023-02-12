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

#[derive(Debug, Default)]
pub struct SceneNode {
    pub global_position: Vec3,
    pub aabb: Aabb,
}

impl SceneNode {
    fn new(global_position: Vec3, aabb: Aabb) -> Self {
        Self {
            global_position,
            aabb,
        }
    }
}

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .init_resource::<AssetsLoading>()
            .add_state(AssetsState::Loading)
            .add_system_set(
                SystemSet::on_update(AssetsState::AllLoaded).with_system(add_scene_nodes),
            )
            .add_system_set(
                SystemSet::on_update(AssetsState::Loading).with_system(check_assets_ready),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AssetsState {
    Loading,
    AllLoaded,
}

fn add_scene_nodes(
    mut commands: Commands,
    scenes: Query<Entity, (With<Handle<Scene>>, Without<AabbParsed>)>,
    children: Query<&Children>,
    existing_meshes: Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    mut aabbs: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    global_transforms: Query<&GlobalTransform>,
) -> () {
    for scene in scenes.iter() {
        let mut scene_nodes: HashMap<Entity, SceneNode> = HashMap::new();
        // println!("Parsing scene: {:?}", scene);
        get_all_meshes_from_children(
            &mut commands,
            scene,
            &children,
            &existing_meshes,
            &mut scene_nodes,
            &global_transforms,
        );

        // # TODO: For testing, replace with a proper flag to ensure
        // scene is completely loaded before parsing.
        if !scene_nodes.is_empty() {
            println!("Scene AABBs: {:#?}", scene_nodes);
            //     commands.entity(scene).insert(AabbParsed);
            //     for scene_node in scene_nodes.into_values() {
            //         let aabb = scene_node.aabb.unwrap_or(Aabb {
            //             center: Vec3A::ZERO,
            //             half_extents: Vec3A::ZERO,
            //         });

            //         commands
            //             .spawn(PbrBundle {
            //                 mesh: meshes.add(Mesh::from(box_mesh_from_aabb(&aabb))),
            //                 transform: Transform {
            //                     translation: Vec3::from(aabb.center),
            //                     ..Default::default()
            //                 },
            //                 material: materials.add(StandardMaterial {
            //                     base_color: Color::Rgba {
            //                         red: 1.0,
            //                         green: 0.0,
            //                         blue: 0.0,
            //                         alpha: 0.5,
            //                     },
            //                     emissive: Color::Rgba {
            //                         red: 1.0,
            //                         green: 0.0,
            //                         blue: 0.0,
            //                         alpha: 0.5,
            //                     },
            //                     ..Default::default()
            //                 }),
            //                 ..Default::default()
            //             })
            //             .insert(Name::from("debug-cube"));
            //     }
        }
    }
}

fn get_all_meshes_from_children<'a>(
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
            // commands.entity(*child).insert(Marker);

            match get_scene_node(*child, aabbs, global_transforms) {
                Ok(scene_node) => scene_nodes.insert(*child, scene_node),
                Err(_) => None,
            };

            scene_nodes = get_all_meshes_from_children(
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

fn get_scene_node(
    entity: Entity,
    aabbs: &Query<(Entity, &Aabb), (With<Handle<Mesh>>, Without<Marker>)>,
    global_transforms: &Query<&GlobalTransform>,
) -> Result<SceneNode, QueryEntityError> {
    let global_transform = global_transforms.get(entity)?;
    let (entity, aabb) = aabbs.get(entity)?;
    println!("Global transform: {:?}", global_transform);
    Ok(SceneNode {
        global_position: global_transform.translation(),
        aabb: aabb.clone(),
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

#[derive(Resource)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

impl Default for AssetsLoading {
    fn default() -> Self {
        AssetsLoading(vec![])
    }
}

fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut assets_load_state: ResMut<State<AssetsState>>,
) {
    match server.get_group_load_state(loading.0.iter().map(|h| h.id)) {
        LoadState::Failed => {
            println!("Uh-oh");
        }
        LoadState::Loaded => {
            println!("Loaded all scenes.");
            assets_load_state.push(AssetsState::AllLoaded).unwrap();
        }
        _ => {
            println!("Not fully loaded yet.");
        }
    }
}
