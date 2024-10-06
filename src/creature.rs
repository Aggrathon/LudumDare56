use avian2d::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use enum_iterator::{all, Sequence};

use crate::utils::StateLocalSpawner;

const MAX_ANGULAR_VELOCITY: f32 = 15.0;
const ARM_WIDTH: f32 = 10.0;
const EYE_RADIUS: f32 = 5.0;

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
                    on_spread_control,
                    switch_main,
                ),
            )
            .init_gizmo_group::<ArmGizmos>();
    }
}

#[derive(Component, Clone, Copy, Default, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum Creature {
    #[default]
    Normal,
    Bouncy,
    Explosive,
    Heavy,
}

impl Creature {
    pub fn color(self) -> Color {
        match self {
            Creature::Normal => Color::srgb(0.6, 0.0, 0.8),
            Creature::Bouncy => Color::srgb(0.0, 0.8, 0.0),
            Creature::Explosive => Color::srgb(0.8, 0.3, 0.0),
            Creature::Heavy => Color::srgb(0.1, 0.1, 0.1),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Creature::Heavy => 22.0,
            Creature::Bouncy => 18.0,
            _ => 20.0,
        }
    }

    pub fn density(self) -> f32 {
        match self {
            Creature::Heavy => 2.0,
            _ => 1.0,
        }
    }

    pub fn jump(self) -> f32 {
        match self {
            Creature::Heavy => 300.0,
            Creature::Bouncy => 400.0,
            _ => 350.0,
        }
    }

    pub fn force(self) -> f32 {
        match self {
            Creature::Explosive => 250.0,
            _ => 150.0,
        }
    }

    pub fn speed(self) -> f32 {
        120.0
    }

    pub fn bounciness(self) -> f32 {
        match self {
            Creature::Bouncy => 0.8,
            _ => 0.4,
        }
    }
}

#[derive(Resource)]
pub struct CreatureAssets {
    map: HashMap<Creature, (Handle<Mesh>, Handle<ColorMaterial>)>,
    eye_mesh: Handle<Mesh>,
    eye_material: Handle<ColorMaterial>,
    mouth_sprite: Handle<Image>,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct ArmGizmos {}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut config_store: ResMut<GizmoConfigStore>,
    asset_server: Res<AssetServer>,
) {
    let assets = CreatureAssets {
        map: all::<Creature>()
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
        eye_mesh: meshes.add(Circle::new(EYE_RADIUS)),
        eye_material: materials.add(Color::WHITE),
        // tongue_sprite: asset_server.load("sprites/tongue.png"),
        mouth_sprite: asset_server.load("sprites/mouth.png"),
    };
    commands.insert_resource(assets);
    config_store.config_mut::<ArmGizmos>().0.line_width = ARM_WIDTH;
}

#[derive(Component, Clone, Copy)]
struct Grounded(u32);

#[derive(Component, Clone, Copy)]
pub struct MainCreature {}

#[derive(Component, Clone, Copy)]
struct Controlled {}

#[derive(Component, Clone, Copy)]
struct SpreadControl {}

#[derive(Component, Clone, Copy)]
struct Mouth {}

fn mouth(radius: f32, assets: &Res<CreatureAssets>) -> impl Bundle {
    (
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(radius * 0.8)),
                ..default()
            },
            texture: assets.mouth_sprite.clone(),
            transform: Transform::from_xyz(0.0, -radius * 0.35, 1.05),
            ..default()
        },
        Mouth {},
    )
}

impl Creature {
    pub fn spawn(
        commands: &mut StateLocalSpawner<'_, '_>,
        x: f32,
        y: f32,
        species: Creature,
        controlled: bool,
        assets: &Res<CreatureAssets>,
    ) -> Entity {
        let (mesh, material) = assets
            .map
            .get(&species)
            .expect("All creature assets should have been initialised in the setup");
        let mut ec = commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            },
            RigidBody::Dynamic,
            ColliderDensity(species.density()),
            Collider::circle(species.radius()),
            Restitution::new(species.bounciness()),
            Friction::new(1.5),
            species,
            Grounded(0),
        ));
        if controlled {
            ec.insert((MainCreature {}, SpreadControl {}));
        }
        ec.with_children(|cb| {
            cb.spawn(MaterialMesh2dBundle {
                mesh: assets.eye_mesh.clone().into(),
                material: assets.eye_material.clone(),
                transform: Transform::from_xyz(species.radius() * 0.4, species.radius() * 0.4, 1.1),
                ..default()
            });
            cb.spawn(MaterialMesh2dBundle {
                mesh: assets.eye_mesh.clone().into(),
                material: assets.eye_material.clone(),
                transform: Transform::from_xyz(
                    -species.radius() * 0.4,
                    species.radius() * 0.4,
                    1.1,
                ),
                ..default()
            });
        });
        ec.id()
    }
}

fn jump(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut LinearVelocity, &Creature, &Grounded), With<Controlled>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        for (mut linear_velocity, creature, grounded) in &mut creatures {
            if grounded.0 > 0 {
                linear_velocity.y += creature.jump();
                // TODO FX
            } else {
                linear_velocity.y += creature.jump() * 0.05;
            }
        }
    }
}

fn explode(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut LinearVelocity, &GlobalTransform, &Creature)>,
    joints: Query<(Entity, &FixedJoint)>,
    controls: Query<(Entity, Option<&MainCreature>), (With<Controlled>, With<Creature>)>,
    mouths: Query<(Entity, &Parent), With<Mouth>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        for (entity, joint) in joints.iter() {
            if let Ok((_, transform1, creature1)) = creatures.get(joint.entity1) {
                if let Ok((_, transform2, creature2)) = creatures.get(joint.entity2) {
                    commands.entity(entity).despawn();
                    let dir = transform2.translation().xy() - transform1.translation().xy();
                    let dir = dir.normalize() * (creature1.force() + creature2.force());
                    creatures.get_mut(joint.entity1).unwrap().0 .0 -= dir;
                    creatures.get_mut(joint.entity2).unwrap().0 .0 += dir;
                    // TODO FX
                }
            }
        }
        for (e, o) in controls.iter() {
            commands.entity(e).remove::<Controlled>();
            if o.is_some() {
                commands.entity(e).insert(SpreadControl {});
            }
        }
        for (e, p) in mouths.iter() {
            commands.entity(p.get()).remove_children(&[e]);
            commands.entity(e).despawn();
        }
    }
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut creatures: Query<(&mut AngularVelocity, &Creature, &Grounded), With<Controlled>>,
) {
    let delta_time = time.delta_seconds();
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    if left & !right {
        for (mut angular_velocity, creature, grounded) in &mut creatures {
            let delta = delta_time * if grounded.0 > 0 { 1.0 } else { 0.5 } * creature.speed();
            angular_velocity.0 = MAX_ANGULAR_VELOCITY.min(angular_velocity.0 + delta);
        }
    } else if right & !left {
        for (mut angular_velocity, creature, grounded) in &mut creatures {
            let delta = delta_time * if grounded.0 > 0 { 1.0 } else { 0.5 } * creature.speed();
            angular_velocity.0 = (-MAX_ANGULAR_VELOCITY).max(angular_velocity.0 - delta);
        }
    }
}

fn on_collision_enter(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    creatures: Query<(&GlobalTransform, &Creature, Option<&Controlled>)>,
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
                let (t1, c1, o1) = creatures.get(*e1).unwrap();
                let (t2, c2, o2) = creatures.get(*e2).unwrap();
                commands.spawn(
                    FixedJoint::new(*e1, *e2)
                        .with_compliance(0.00001)
                        .with_local_anchor_1(
                            t1.transform_point(t2.translation()).truncate().normalize()
                                * c1.radius(),
                        )
                        .with_local_anchor_2(
                            t2.transform_point(t1.translation()).truncate().normalize()
                                * c2.radius(),
                        ),
                );
                if o1.is_some() && o2.is_none() {
                    commands.entity(*e2).insert(SpreadControl {});
                } else if o2.is_some() && o1.is_none() {
                    commands.entity(*e1).insert(SpreadControl {});
                }
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
                    v1 + dir * (c1.radius() * 0.75),
                    v2 - dir * (c2.radius() * 0.75),
                    c1.color(),
                    c2.color(),
                );
            }
        }
    }
}

fn switch_main(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    creatures: Query<(Entity, Option<&MainCreature>, Option<&Controlled>), With<Creature>>,
    mouths: Query<(Entity, &Parent), With<Mouth>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Tab]) {
        let mut skip = true;
        let mut sel_bc = None;
        let mut sel_bu = None;
        let mut sel_ac = None;
        let mut sel_au = None;
        for (e, mc, c) in creatures.iter() {
            if c.is_some() {
                commands.entity(e).remove::<Controlled>();
            }
            if mc.is_some() {
                commands.entity(e).remove::<MainCreature>();
                skip = false;
            } else if skip {
                if c.is_none() {
                    if sel_au.is_none() {
                        sel_au = Some(e);
                    }
                } else if sel_ac.is_none() {
                    sel_ac = Some(e)
                }
            } else if c.is_none() {
                if sel_bu.is_none() {
                    sel_bu = Some(e);
                }
            } else if sel_bc.is_none() {
                sel_bc = Some(e)
            }
        }
        if let Some(e) = sel_au.or(sel_bu).or(sel_ac).or(sel_bc) {
            commands
                .entity(e)
                .insert((MainCreature {}, SpreadControl {}));
        }
        for (e, p) in mouths.iter() {
            commands.entity(p.get()).remove_children(&[e]);
            commands.entity(e).despawn();
        }
    }
}

fn on_spread_control(
    mut commands: Commands,
    spread: Query<(Entity, &Creature), Added<SpreadControl>>,
    joints: Query<&FixedJoint>,
    controls: Query<(), (With<Controlled>, With<Creature>)>,
    assets: Res<CreatureAssets>,
) {
    for (entity, creature) in spread.iter() {
        commands
            .entity(entity)
            .remove::<SpreadControl>()
            .insert(Controlled {})
            .with_children(|cb| {
                cb.spawn(mouth(creature.radius(), &assets));
            });
        for joint in joints.iter() {
            if joint.entity1 == entity {
                if !controls.contains(joint.entity2) {
                    commands.entity(joint.entity2).insert(SpreadControl {});
                }
            } else if joint.entity2 == entity && !controls.contains(joint.entity1) {
                commands.entity(joint.entity1).insert(SpreadControl {});
            }
        }
    }
}
