mod audio;
mod creature;
mod levels;
mod objects;
mod ui;
mod utils;

use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(40.0),
            RngPlugin::default(),
            creature::CreaturePlugin,
            objects::ObjectPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
            levels::LevelPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.25)))
        .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 50.0))
        .run();
}
