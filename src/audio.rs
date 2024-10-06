use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_turborand::prelude::*;

use crate::utils::StateLocalSpawner;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, sound_event)
            .add_event::<Sounds>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Event)]
pub enum Sounds {
    Click,
    Grunt,
    Hello,
    Glass,
    Music,
}

#[derive(Resource)]
struct AudioAssets {
    map: HashMap<Sounds, Vec<Handle<AudioSource>>>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut map = HashMap::new();
    map.insert(
        Sounds::Click,
        vec![
            asset_server.load("audio/click01.ogg"),
            asset_server.load("audio/click02.ogg"),
            asset_server.load("audio/click03.ogg"),
            asset_server.load("audio/click04.ogg"),
        ],
    );
    map.insert(
        Sounds::Hello,
        vec![
            asset_server.load("audio/hello01.ogg"),
            asset_server.load("audio/hello02.ogg"),
            asset_server.load("audio/hello03.ogg"),
            asset_server.load("audio/hello04.ogg"),
        ],
    );
    map.insert(
        Sounds::Grunt,
        vec![
            asset_server.load("audio/grunt01.ogg"),
            asset_server.load("audio/grunt02.ogg"),
            asset_server.load("audio/grunt03.ogg"),
            asset_server.load("audio/grunt04.ogg"),
        ],
    );
    map.insert(
        Sounds::Glass,
        vec![
            asset_server.load("audio/glass01.ogg"),
            asset_server.load("audio/glass02.ogg"),
            asset_server.load("audio/glass03.ogg"),
            asset_server.load("audio/glass04.ogg"),
        ],
    );
    commands.insert_resource(AudioAssets { map });
}

// pub fn sound(sound: Sounds, assets: Res<AudioAssets>, mut rng: ResMut<GlobalRng>) -> impl Bundle {
//     if let Some(source) = assets.map.get(&sound).and_then(|a| rng.sample(a)).cloned() {
//         let settings = PlaybackSettings {
//             mode: PlaybackMode::Once,
//             speed: if sound != Sounds::Music {
//                 rng.f32() * 0.3 + 0.9
//             } else {
//                 1.0
//             },
//             ..default()
//         };
//         AudioBundle { source, settings }
//     } else {
//         AudioBundle::default()
//     }
// }

fn sound_event(
    commands: Commands,
    mut sounds: EventReader<Sounds>,
    assets: Res<AudioAssets>,
    mut rng: ResMut<GlobalRng>,
) {
    let mut cmd = StateLocalSpawner(commands);
    for sound in sounds.read() {
        if let Some(source) = assets.map.get(sound).and_then(|a| rng.sample(a)).cloned() {
            let settings = PlaybackSettings {
                mode: PlaybackMode::Once,
                speed: if *sound != Sounds::Music {
                    rng.f32() * 0.3 + 0.9
                } else {
                    1.0
                },
                volume: Volume::new(0.5),
                ..default()
            };
            cmd.spawn(AudioBundle { source, settings });
        }
    }
}
