use avian2d::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use enum_iterator::{all, Sequence};

const MAX_ANGULAR_VELOCITY: f32 = 15.0;
const ARM_WIDTH: f32 = 10.0;

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    movement,
                    jump,
                    explode,
                    on_collision_enter,
                    on_collision_exit,
                    arms,
                ),
            )
            .init_gizmo_group::<ArmGizmos>();
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

    pub fn jump(self) -> f32 {
        match self {
            Species::Heavy => 300.0,
            Species::Explosive | Species::Bouncy => 400.0,
            _ => 350.0,
        }
    }

    pub fn force(self) -> f32 {
        match self {
            Species::Explosive => 250.0,
            _ => 150.0,
        }
    }

    pub fn speed(self) -> f32 {
        120.0
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

#[derive(Default, Reflect, GizmoConfigGroup)]
struct ArmGizmos {}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut config_store: ResMut<GizmoConfigStore>,
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
    config_store.config_mut::<ArmGizmos>().0.line_width = ARM_WIDTH;
}

#[derive(Component, Clone, Copy)]
pub struct Creature(Species);

#[derive(Component, Clone, Copy)]
pub struct Grounded(u32);

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
            Friction::new(1.1).with_dynamic_coefficient(0.9),
            Creature(species),
            Grounded(0),
        )
    }
}

fn jump(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut LinearVelocity, &Creature, &Grounded)>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        for (mut linear_velocity, creature, grounded) in &mut creatures {
            if grounded.0 > 0 {
                linear_velocity.y += creature.0.jump();
                // TODO FX
            } else {
                linear_velocity.y += creature.0.jump() * 0.05;
            }
        }
    }
}

fn explode(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut LinearVelocity, &GlobalTransform, &Creature)>,
    joints: Query<(Entity, &FixedJoint)>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown, KeyCode::Space]) {
        for (entity, joint) in joints.iter() {
            if let Ok((_, transform1, creature1)) = creatures.get(joint.entity1) {
                if let Ok((_, transform2, creature2)) = creatures.get(joint.entity2) {
                    commands.entity(entity).despawn();
                    let dir = transform2.translation().xy() - transform1.translation().xy();
                    let dir = dir.normalize() * (creature1.0.force() + creature2.0.force());
                    creatures.get_mut(joint.entity1).unwrap().0 .0 -= dir;
                    creatures.get_mut(joint.entity2).unwrap().0 .0 += dir;
                    // TODO FX
                }
            }
        }
    }
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut AngularVelocity, &Creature, &Grounded)>,
) {
    let delta_time = time.delta_seconds();
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    if left & !right {
        for (mut angular_velocity, creature, grounded) in &mut creatures {
            let delta = delta_time * if grounded.0 > 0 { 1.0 } else { 0.5 } * creature.0.speed();
            angular_velocity.0 = MAX_ANGULAR_VELOCITY.min(angular_velocity.0 + delta);
        }
    } else if right & !left {
        for (mut angular_velocity, creature, grounded) in &mut creatures {
            let delta = delta_time * if grounded.0 > 0 { 1.0 } else { 0.5 } * creature.0.speed();
            angular_velocity.0 = (-MAX_ANGULAR_VELOCITY).max(angular_velocity.0 - delta);
        }
    }
}

fn on_collision_enter(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    creatures: Query<(&GlobalTransform, &Creature)>,
    joints: Query<&FixedJoint>,
    mut groundeds: Query<&mut Grounded>,
) {
    'outer: for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if creatures.contains(*e1) {
            if creatures.contains(*e2) {
                for joint in joints.iter() {
                    if joint.entity1 == *e1 && joint.entity2 == *e2 {
                        continue 'outer;
                    }
                    if joint.entity1 == *e2 && joint.entity2 == *e1 {
                        continue 'outer;
                    }
                }
                let (t1, c1) = creatures.get(*e1).unwrap();
                let (t2, c2) = creatures.get(*e1).unwrap();
                commands.spawn(
                    FixedJoint::new(*e1, *e2)
                        .with_compliance(0.00001)
                        .with_local_anchor_1(
                            t1.transform_point(t2.translation()).truncate().normalize()
                                * c1.0.radius(),
                        )
                        .with_local_anchor_2(
                            t2.transform_point(t1.translation()).truncate().normalize()
                                * c2.0.radius(),
                        ),
                );
                // TODO FX
            } else {
                groundeds
                    .get_mut(*e1)
                    .expect("Creatures should have `Grounded`")
                    .0 += 1;
            }
        } else if creatures.contains(*e2) {
            groundeds
                .get_mut(*e2)
                .expect("Creatures should have `Grounded`")
                .0 += 1;
        }
    }
}

fn on_collision_exit(
    mut collision_event_reader: EventReader<CollisionEnded>,
    creatures: Query<(), With<Creature>>,
    mut groundeds: Query<&mut Grounded>,
) {
    for CollisionEnded(e1, e2) in collision_event_reader.read() {
        if creatures.contains(*e1) {
            if !creatures.contains(*e2) {
                groundeds
                    .get_mut(*e1)
                    .expect("Creatures should have `Grounded`")
                    .0 -= 1;
            }
        } else if creatures.contains(*e2) {
            groundeds
                .get_mut(*e2)
                .expect("Creatures should have `Grounded`")
                .0 -= 1;
        }
    }
}

fn arms(
    joints: Query<&FixedJoint>,
    transforms: Query<(&GlobalTransform, &Creature)>,
    mut gizmos: Gizmos<ArmGizmos>,
) {
    for joint in joints.iter() {
        if let Ok((gt1, c1)) = transforms.get(joint.entity1) {
            if let Ok((gt2, c2)) = transforms.get(joint.entity2) {
                let v1 = gt1.translation().xy();
                let v2 = gt2.translation().xy();
                let dir = (v2 - v1).normalize();

                gizmos.line_gradient_2d(
                    v1 + dir * (c1.0.radius() * 0.5),
                    v2 - dir * (c2.0.radius() * 0.5),
                    c1.0.color(),
                    c2.0.color(),
                );
            }
        }
    }
}
