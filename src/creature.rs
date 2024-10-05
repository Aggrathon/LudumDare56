use avian2d::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use enum_iterator::{all, Sequence};

const MAX_ANGULAR_VELOCITY: f32 = 20.0;
const JUMP_COOLDOWN: f64 = 1.0;

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (movement, jump));
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Species {
    #[default]
    Normal,
    Bouncy,
    Explosive,
    Heavy,
}

impl Species {
    pub fn color(self) -> Color {
        match self {
            Species::Normal => Color::srgb(0.6, 0.0, 0.8),
            Species::Bouncy => Color::srgb(0.0, 0.8, 0.0),
            Species::Explosive => Color::srgb(0.8, 0.3, 0.0),
            Species::Heavy => Color::srgb(0.1, 0.1, 0.1),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Species::Heavy => 22.0,
            Species::Bouncy => 18.0,
            _ => 20.0,
        }
    }

    pub fn density(self) -> f32 {
        match self {
            Species::Heavy => 2.0,
            _ => 1.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            Species::Heavy => 10.0,
            Species::Explosive | Species::Bouncy => 15.0,
            _ => 12.0,
        }
    }

    pub fn bounciness(self) -> f32 {
        match self {
            Species::Bouncy => 0.7,
            _ => 0.4,
        }
    }
}

#[derive(Resource)]
pub struct CreatureAssets {
    map: HashMap<Species, (Handle<Mesh>, Handle<ColorMaterial>)>,
}

#[derive(Resource)]
struct JumpCooldown(f64);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let assets = CreatureAssets {
        map: all::<Species>()
            .map(|species| {
                (
                    species,
                    (
                        meshes.add(Circle::new(species.radius())),
                        materials.add(species.color()),
                    ),
                )
            })
            .collect(),
    };
    commands.insert_resource(assets);
    commands.insert_resource(JumpCooldown(0.0));
}

#[derive(Component, Clone, Copy)]
pub struct Creature(Species);

impl Creature {
    pub fn spawn(x: f32, y: f32, species: Species, assets: &Res<CreatureAssets>) -> impl Bundle {
        // TODO add eyes
        let (mesh, material) = assets
            .map
            .get(&species)
            .expect("All creature assets should have been initialised in the setup");
        (
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            ColliderDensity(species.density()),
            Collider::circle(species.radius()),
            Restitution::new(species.bounciness()),
            Creature(species),
        )
    }
}

fn jump(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut LinearVelocity, &Creature)>,
    mut jump_cooldown: ResMut<JumpCooldown>,
) {
    // TODO check for grounded
    let jump = jump_cooldown.0 < time.elapsed_seconds_f64()
        && keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space]);
    if jump {
        jump_cooldown.0 = time.elapsed_seconds_f64() + JUMP_COOLDOWN;
        for (mut linear_velocity, creature) in &mut creatures {
            linear_velocity.y += 30.0 * creature.0.speed();
        }
    }
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut AngularVelocity, &Creature)>,
) {
    let delta_time = time.delta_seconds();
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    if left & !right {
        for (mut angular_velocity, creature) in &mut creatures {
            angular_velocity.0 =
                MAX_ANGULAR_VELOCITY.min(angular_velocity.0 + delta_time * creature.0.speed());
        }
    } else if right & !left {
        for (mut angular_velocity, creature) in &mut creatures {
            angular_velocity.0 =
                (-MAX_ANGULAR_VELOCITY).max(angular_velocity.0 - delta_time * creature.0.speed());
        }
    }
}
