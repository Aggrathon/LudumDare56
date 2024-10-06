use crate::creature::{Creature, CreatureAssets};
use crate::objects::{
    background, camera, door, plank, spawn_exit, spawn_glass, spawn_pressure_plate, wall,
    PressurePlateEvent,
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
        .add_systems(OnEnter(Level::Tutorial3), setup_tutorial3)
        .add_systems(OnEnter(Level::Tutorial4), setup_tutorial4)
        .add_systems(OnEnter(Level::Tutorial5), setup_tutorial5)
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
    Tutorial3,
    Tutorial4,
    Tutorial5,
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
            _ => {}
        }
    }
    for signal in signals.read() {
        match signal {
            Signal::NextLevel => next_state.set(state.get().next().unwrap_or_default()),
            Signal::RestartLevel => next_state.set(*state.get()),
            _ => {}
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

    let e1 = Creature::spawn(&mut commands, -60.0, 50.0, Creature::Normal, true, &assets);
    let e2 = commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(-60.0, -60.0, 1.0)),
            RigidBody::Static,
        ))
        .id();
    commands.spawn(FixedJoint::new(e1, e2).with_compliance(0.0001));
    commands.spawn(plank(Vec2::new(-100.0, -75.0), Vec2::new(-0.0, -75.0)));
}

fn setup_test_level(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    cmd.spawn(plank(Vec2::new(300.0, 10.0), Vec2::new(450.0, -10.0)));
    cmd.spawn(plank(Vec2::new(-300.0, 10.0), Vec2::new(-450.0, -10.0)));
    cmd.spawn(plank(Vec2::new(250.0, 10.0), Vec2::new(300.0, 10.0)));
    cmd.spawn(plank(Vec2::new(-250.0, 10.0), Vec2::new(-300.0, 10.0)));

    spawn_exit(&mut cmd, Vec2::new(275.0, 10.0), 60.0, 0.0);
    spawn_pressure_plate(
        &mut cmd,
        Signal::Custom(0),
        Vec2::new(-275.0, -275.0),
        60.0,
        0.0,
    );

    spawn_glass(&mut cmd, Vec2::new(250.0, -275.), 80.);

    spawn_sign(
        &mut cmd,
        "Press N to go to the next level",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );

    let d = 30.0;
    Creature::spawn(&mut cmd, -d * 3.0, 0.0, Creature::Normal, true, &assets);
    Creature::spawn(&mut cmd, -d * 1.0, 0.0, Creature::Explosive, false, &assets);
    Creature::spawn(&mut cmd, d * 1.0, 0.0, Creature::Bouncy, false, &assets);
    Creature::spawn(&mut cmd, d * 3.0, 0.0, Creature::Heavy, false, &assets);
}

fn setup_tutorial1(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    spawn_exit(&mut cmd, Vec2::new(275.0, -275.0), 60.0, 0.0);

    spawn_sign(
        &mut cmd,
        "Tony dreams of a break\nfrom the dark and the grey.",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut cmd,
        "Press A / D / ← / → to roll.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );
    Creature::spawn(&mut cmd, -150., 0.0, Creature::Normal, true, &assets);
}

fn setup_tutorial2(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    cmd.spawn(plank(Vec2::new(150.0, -150.0), Vec2::new(300.0, -150.0)));
    cmd.spawn(plank(Vec2::new(300.0, -50.0), Vec2::new(450.0, -50.0)));
    spawn_exit(&mut cmd, Vec2::new(400.0, -50.0), 60.0, 0.0);

    spawn_sign(
        &mut cmd,
        "He needs a holiday, preferrably\nsomewhere bright and warm!",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut cmd,
        "Press W / ↑ to jump.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );
    Creature::spawn(&mut cmd, -150., 0.0, Creature::Normal, true, &assets);
}

fn setup_tutorial3(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    cmd.spawn(wall(Vec2::new(175.0, 275.0), Vec2::new(225.0, -175.0)));
    spawn_glass(&mut cmd, Vec2::new(200.0, -275.), 100.);

    spawn_exit(&mut cmd, Vec2::new(275.0, -275.0), 60.0, 0.0);

    spawn_sign(
        &mut cmd,
        "Ricky has always been a good\nfriend, maybe he can help?",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut cmd,
        "Press R to restart.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );

    Creature::spawn(&mut cmd, -50., 0.0, Creature::Normal, true, &assets);
    Creature::spawn(&mut cmd, -250., 0.0, Creature::Heavy, false, &assets);
}

fn setup_tutorial4(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    cmd.spawn(wall(Vec2::new(175.0, -100.0), Vec2::new(225.0, -275.0)));

    spawn_exit(&mut cmd, Vec2::new(275.0, -275.0), 60.0, 0.0);

    spawn_sign(
        &mut cmd,
        "Issy has a hot temperament,\nwhich is scary at times.",
        Vec2::new(-150., 120.),
        Vec2::new(150., 60.),
        &text_styles,
    );
    spawn_sign(
        &mut cmd,
        "Press S / ↓ to shove\neveryone away.",
        Vec2::new(-150., 0.),
        Vec2::new(150., -80.),
        &text_styles,
    );

    Creature::spawn(&mut cmd, -50., 0.0, Creature::Normal, true, &assets);
    Creature::spawn(&mut cmd, -250., 0., Creature::Explosive, false, &assets);
}

fn setup_tutorial5(commands: Commands, assets: Res<CreatureAssets>, text_styles: Res<TextStyles>) {
    let mut cmd = StateLocalSpawner(commands);
    cmd.spawn(camera());

    cmd.spawn(wall(Vec2::new(-500.0, 325.0), Vec2::new(500.0, 275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, -325.0), Vec2::new(500.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-500.0, 275.0), Vec2::new(-450.0, -275.0)));
    cmd.spawn(wall(Vec2::new(500.0, 275.0), Vec2::new(450.0, -275.0)));

    cmd.spawn(wall(Vec2::new(175.0, 275.0), Vec2::new(225.0, -275.0)));
    cmd.spawn(wall(Vec2::new(-175.0, 275.0), Vec2::new(-225.0, -175.0)));
    cmd.spawn(door(0, Vec2::new(-200., -275.), 100.));

    spawn_pressure_plate(&mut cmd, Signal::Door(0), Vec2::new(350., -275.), 60., 0.0);
    spawn_exit(&mut cmd, Vec2::new(-350.0, -275.0), 60.0, 0.0);

    spawn_sign(
        &mut cmd,
        "Elly is eager to help, but\nrequires detailed instructions.",
        Vec2::new(-150., 100.),
        Vec2::new(150., 20.),
        &text_styles,
    );
    spawn_sign(
        &mut cmd,
        "Press Space to switch creature.",
        Vec2::new(-150., -20.),
        Vec2::new(150., -60.),
        &text_styles,
    );

    Creature::spawn(&mut cmd, 0., 0.0, Creature::Normal, true, &assets);
    Creature::spawn(&mut cmd, 250., 0., Creature::Bouncy, false, &assets);
}
