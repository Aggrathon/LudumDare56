use bevy::prelude::*;

use crate::utils::StateLocalSpawner;

const SIGN_COLOR_TEXT: Color = Color::WHITE;
const SIGN_COLOR_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_BG_COLOR: Color = Color::WHITE; //Color::srgb(0.25, 0.35, 0.25);
const BUTTON_TEXT_COLOR: Color = Color::BLACK;
const BUTTON_FRAME_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const BUTTON_FRAME_HOVER: Color = Color::BLACK;
const BUTTON_FRAME_PRESSED: Color = Color::WHITE;
const BUTTON_FRAME_WIDTH: Val = Val::Px(5.0);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, button_system)
            .add_event::<Signal>();
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub enum Signal {
    NextLevel,
    #[expect(dead_code)]
    RestartLevel,
    Custom(u16),
}

#[derive(Component, Debug, Clone, Copy)]
struct ButtonSignal(Signal, bool);

#[derive(Resource)]
pub struct TextStyles {
    pub sign_text: TextStyle,
    pub button_text: TextStyle,
    pub title_text: TextStyle,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Comfortaa-Regular.ttf");
    commands.insert_resource(TextStyles {
        sign_text: TextStyle {
            font: font.clone(),
            font_size: 20.0,
            color: SIGN_COLOR_TEXT,
        },
        button_text: TextStyle {
            font: font.clone(),
            font_size: 32.0,
            color: BUTTON_TEXT_COLOR,
        },
        title_text: TextStyle {
            font: font.clone(),
            font_size: 72.0,
            color: SIGN_COLOR_TEXT,
        },
    });
}

fn button_system(
    mut interactions: Query<
        (&Interaction, &mut BorderColor, &mut ButtonSignal),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: EventWriter<Signal>,
) {
    for (interaction, mut color, mut signal) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                color.0 = BUTTON_FRAME_PRESSED;
                signal.1 = true;
            }
            Interaction::Hovered => {
                color.0 = BUTTON_FRAME_HOVER;
                if signal.1 {
                    event.send(signal.0);
                }
                signal.1 = false;
            }
            Interaction::None => {
                color.0 = BUTTON_FRAME_COLOR;
                signal.1 = false;
            }
        }
    }
}

pub fn spawn_sign(
    commands: &mut StateLocalSpawner<'_, '_>,
    text: &str,
    topleft: Vec2,
    bottomright: Vec2,
    text_styles: &Res<TextStyles>,
) {
    commands
        .spawn((Text2dBundle {
            text: Text::from_section(text, text_styles.sign_text.clone())
                .with_justify(JustifyText::Center),
            transform: Transform::from_translation(topleft.midpoint(bottomright).extend(-0.2)),
            ..default()
        },))
        .with_children(|cb| {
            cb.spawn((SpriteBundle {
                sprite: Sprite {
                    color: SIGN_COLOR_BG,
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -0.05)
                    .with_scale((bottomright - topleft).abs().extend(1.0)),
                ..default()
            },));
        });
}

pub fn spawn_button(
    commands: &mut ChildBuilder,
    signal: Signal,
    text: &str,
    width: Val,
    height: Val,
    text_styles: Res<TextStyles>,
) {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width,
                    height,
                    border: UiRect::all(BUTTON_FRAME_WIDTH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(BUTTON_FRAME_COLOR),
                border_radius: BorderRadius::MAX,
                background_color: BUTTON_BG_COLOR.into(),
                ..default()
            },
            ButtonSignal(signal, false),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                text_styles.button_text.clone(),
            ));
        });
}
