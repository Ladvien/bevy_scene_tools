use bevy::prelude::*;

pub struct SceneToolsPlugin;

impl Plugin for SceneToolsPlugin {
    fn build(&self, app: &mut App) {
        let app = app.add_system(add_scene_aabbs);
    }
}

fn add_scene_aabbs(query: Query<&Handle<Scene>>) -> () {
    for scene in query.iter() {
        println!("{:?}", scene);
    }
}
