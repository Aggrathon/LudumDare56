use crate::creature::{Creature, CreatureAssets, Species};
use crate::objects::{plank, wall};
use crate::utils::{IdentityTransitionsPlugin, StateLocalPlugin, StateLocalSpawner};
use bevy::prelude::*;
use enum_iterator::Sequence;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            IdentityTransitionsPlugin::<Level>::default(),
            StateLocalPlugin::<Level>::default(),
        ))
        .init_state::<Level>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(Level::Level1), test_level)
        .add_systems(Update, input);
    }
}

#[derive(Clone, Copy, Default, States, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Level {
    #[default]
    Menu,
    Level1,
}

fn setup(mut state: ResMut<NextState<Level>>) {
    // TODO Menu
    // TODO Tutorials
    // TODO Levels
    state.set(Level::Level1);
}

fn test_level(commands: Commands, assets: Res<CreatureAssets>) {
    let mut commands = StateLocalSpawner(commands);
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

fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<Level>>,
    state: Res<State<Level>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyR, KeyCode::Home]) {
        next_state.set(*state.get());
    } else if keyboard_input.any_just_pressed([KeyCode::KeyN, KeyCode::End]) {
        next_state.set(state.get().next().unwrap_or_default());
    }
}
