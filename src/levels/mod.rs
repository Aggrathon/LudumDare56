mod level1;

use bevy::prelude::*;
use enum_iterator::Sequence;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<Level>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(Level::Level1), level1::load_level);
    }
}

#[derive(Clone, Copy, Default, States, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Level {
    #[default]
    Menu,
    Level1,
}

pub fn setup(mut state: ResMut<NextState<Level>>) {
    // TODO Menu
    // TODO Tutorials
    // TODO Levels
    state.set(Level::Level1);
}
