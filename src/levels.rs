use crate::creature::{Creature, CreatureAssets, Species};
use crate::objects::{
    background, camera, plank, spawn_exit, spawn_pressure_plate, wall, PressurePlateEvent,
};
use crate::ui::{spawn_button, spawn_sign, Signal, TextStyles};
use crate::utils::{IdentityTransitionsPlugin, StateLocalPlugin, StateLocalSpawner};
use avian2d::math::PI;
use avian2d::prelude::*;
use bevy::prelude::*;
use enum_iterator::Sequence;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            IdentityTransitionsPlugin::<Level>::default(),
            StateLocalPlugin::<Level>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(Level::Test), setup_test_level)
        .add_systems(OnEnter(Level::Menu), setup_main_menu)
        .add_systems(OnEnter(Level::Tutorial1), setup_tutorial1)
        .add_systems(OnEnter(Level::Tutorial2), setup_tutorial2)
        .add_systems(Update, (handle_input, level_events))
        .insert_state(Level::Loading);
    }
}

#[derive(Clone, Copy, Default, States, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Level {
    Loading,
    Test,
    #[default]
    Menu,
    Tutorial1,
    Tutorial2,
    // TODO Tutorials
    // TODO Levels
}

fn setup(mut state: ResMut<NextState<Level>>) {
    #[cfg(debug_assertions)]
    state.set(Level::last().unwrap());
    #[cfg(not(debug_assertions))]
    state.set(Level::Menu);
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<Level>>,
    state: Res<State<Level>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyR, KeyCode::Home]) {
        next_state.set(*state.get());
    } else if keyboard_input.any_just_pressed([KeyCode::KeyN, KeyCode::End]) {
        next_state.set(state.get().next().unwrap_or_default());
    } else if keyboard_input.any_just_pressed([KeyCode::KeyP, KeyCode::Insert]) {
        next_state.set(state.get().previous().unwrap_or_default());
    }
}

fn level_events(
    mut signals: EventReader<Signal>,
    mut events: EventReader<PressurePlateEvent>,
    mut next_state: ResMut<NextState<Level>>,
    state: Res<State<Level>>,
) {
    for PressurePlateEvent(_, signal, pressed) in events.read() {
        match signal {
            Signal::NextLevel => {
                if *pressed {
                    next_state.set(state.get().next().unwrap_or_default())
                }
            }
            Signal::RestartLevel => {
                if *pressed {
                    next_state.set(*state.get())
                }
            }
            Signal::Custom(i) => {
                eprintln!("Pressure plate event not handled: {} = {}", i, pressed);
            }
        }
    }
    for signal in signals.read() {
        match signal {
            Signal::NextLevel => next_state.set(state.get().next().unwrap_or_default()),
            Signal::RestartLevel => next_state.set(*state.get()),
            Signal::Custom(i) => {
                eprintln!("Signal not handled: {}", i);
            }
        }
    }
}

fn setup_main_menu(commands: Commands, text_style: Res<TextStyles>, assets: Res<CreatureAssets>) {
    let mut commands = StateLocalSpawner(commands);
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(150.0),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..default()
    });
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                justify_items: JustifyItems::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|cb| {
            cb.spawn(
                TextBundle::from_section("The Beach Holiday", text_style.title_text.clone())
                    .with_text_justify(JustifyText::Center),
            );
            spawn_button(
                cb,
                Signal::NextLevel,
                "Play",
                Val::Px(200.0),
                Val::Px(80.0),
                text_style,
            );
        });

    commands.spawn(background(
        Color::srgb(0.7, 0.8, 1.0),
        Vec2::new(-400., 200.),
        Vec2::new(400., -200.),
        0.0,
        -2.2,
    ));
    commands.spawn(background(
        Color::srgb(0.4, 0.7, 0.9),
        Vec2::new(-400., -50.),
        Vec2::new(400., -200.),
        0.0,
        -2.1,
    ));
    commands.spawn(background(
        Color::srgb(1.0, 1.0, 0.5),
        Vec2::new(-400., -50.),
        Vec2::new(400., -200.),
        -PI * 0.05,
        -2.,
    ));

    let e1 = Creature::spawn(&mut commands, -60.0, 50.0, Species::Normal, &assets);
    let e2 = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(-40.0, -40.0, 1.0)),
            RigidBody::Static,
        ))
        .id();
    commands.spawn(FixedJoint::new(e1, e2).with_compliance(0.001));
    commands.spawn(plank(Vec2::new(-100.0, -75.0), Vec2::new(-0.0, -75.0)));
}

fn setup_test_level(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut commands = StateLocalSpawner(commands);
    commands.spawn(camera());

    commands.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    commands.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    commands.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    commands.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    commands.spawn(plank(Vec2::new(300.0, 10.0), Vec2::new(450.0, -10.0)));
    commands.spawn(plank(Vec2::new(-300.0, 10.0), Vec2::new(-450.0, -10.0)));
    commands.spawn(plank(Vec2::new(250.0, 10.0), Vec2::new(300.0, 10.0)));
    commands.spawn(plank(Vec2::new(-250.0, 10.0), Vec2::new(-300.0, 10.0)));

    spawn_exit(&mut commands, Vec2::new(275.0, 10.0), 40.0, 0.0);
    spawn_pressure_plate(
        &mut commands,
        Signal::Custom(0),
        Vec2::new(-275.0, -275.0),
        40.0,
        0.0,
    );

    spawn_sign(
        &mut commands,
        "Press N to go to the next level",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );

    let d = 30.0;
    Creature::spawn(&mut commands, -d * 3.0, 0.0, Species::Normal, &assets);
    Creature::spawn(&mut commands, -d * 1.0, 0.0, Species::Explosive, &assets);
    Creature::spawn(&mut commands, d * 1.0, 0.0, Species::Bouncy, &assets);
    Creature::spawn(&mut commands, d * 3.0, 0.0, Species::Heavy, &assets);
}

fn setup_tutorial1(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut commands = StateLocalSpawner(commands);
    commands.spawn(camera());

    commands.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    commands.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    commands.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    commands.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    spawn_exit(&mut commands, Vec2::new(275.0, -275.0), 40.0, 0.0);

    spawn_sign(
        &mut commands,
        "Tony dreams of a break\nfrom the dark and the grey.",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut commands,
        "Use A / D / ← / → to roll.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );
    Creature::spawn(&mut commands, -150., 0.0, Species::Normal, &assets);
}

fn setup_tutorial2(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut commands = StateLocalSpawner(commands);
    commands.spawn(camera());

    commands.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    commands.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    commands.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    commands.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    commands.spawn(plank(Vec2::new(150.0, -150.0), Vec2::new(300.0, -150.0)));
    commands.spawn(plank(Vec2::new(300.0, -50.0), Vec2::new(450.0, -50.0)));
    spawn_exit(&mut commands, Vec2::new(400.0, -50.0), 40.0, 0.0);

    spawn_sign(
        &mut commands,
        "He needs a holiday, preferrably\nsomewhere bright and warm!",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut commands,
        "Use W / ↑ to jump.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );
    Creature::spawn(&mut commands, -150., 0.0, Species::Normal, &assets);
}
