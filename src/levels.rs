use crate::creature::{Creature, CreatureAssets, Species};
use crate::objects::{plank, wall};
use bevy::prelude::*;
use enum_iterator::Sequence;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<Level>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(Level::Level1), test_level);
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

pub fn test_level(mut commands: Commands, assets: Res<CreatureAssets>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    commands.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    commands.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    commands.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    commands.spawn(plank(Vec2::new(300.0, 10.0), Vec2::new(450.0, -10.0)));
    commands.spawn(plank(Vec2::new(-300.0, 10.0), Vec2::new(-450.0, -10.0)));
    commands.spawn(plank(Vec2::new(250.0, 10.0), Vec2::new(300.0, 10.0)));
    commands.spawn(plank(Vec2::new(-250.0, 10.0), Vec2::new(-300.0, 10.0)));

    let d = 30.0;
    Creature::spawn(&mut commands, -d * 3.0, 0.0, Species::Normal, &assets);
    Creature::spawn(&mut commands, -d * 1.0, 0.0, Species::Explosive, &assets);
    Creature::spawn(&mut commands, d * 1.0, 0.0, Species::Bouncy, &assets);
    Creature::spawn(&mut commands, d * 3.0, 0.0, Species::Heavy, &assets);
}
