use bevy::prelude::*;

use bevy_snake::{self, MainPlugin};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Snake at Codemotion!".to_string(),
            width: 300.,
            height: 300.,
            resizable: false,
            cursor_visible: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MainPlugin)
        .run();
}
