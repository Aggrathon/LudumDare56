mod creature;
mod levels;
mod objects;

use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default().with_length_unit(40.0),
            creature::CreaturePlugin,
            levels::LevelPlugin,
            objects::ObjectPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 50.0))
        .run();
}
