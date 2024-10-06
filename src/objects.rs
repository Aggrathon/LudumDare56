use avian2d::prelude::*;
use bevy::prelude::*;

use crate::audio::Sounds;
use crate::creature::{Creature, MainCreature};
use crate::ui::Signal;
use crate::utils::StateLocalSpawner;

const PLANK_THICKNESS: f32 = 25.0;
const SENSOR_THICKNESS: f32 = 6.0;
const EXIT_HEIGHT: f32 = 70.0;
const GLASS_THICKNESS: f32 = 8.0;
const DOOR_THICKNESS: f32 = 12.0;
const CAMERA_SPEED: f32 = 200.0;

const STATIC_COLOR: Color = Color::srgb(0.8, 0.75, 1.0);
const SENSOR_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const EXIT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 0.7);
const GLASS_COLOR: Color = Color::srgba(0.7, 0.75, 1.0, 0.7);
const DOOR_COLOR: Color = Color::srgb(0.35, 0.4, 0.3);

pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                on_pressure_enter,
                on_pressure_exit,
                on_pressure_event,
                glass_collision,
                camera_follow,
            ),
        )
        .add_event::<PressurePlateEvent>();
    }
}

pub fn camera() -> impl Bundle {
    Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(640.0),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..default()
    }
}

fn camera_follow(
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &GlobalTransform, &Camera)>,
    target: Query<&GlobalTransform, With<MainCreature>>,
) {
    if let Ok((mut transform, gt, camera)) = camera.get_single_mut() {
        if let Ok(creature) = target.get_single() {
            if let Some(ndc) = camera.world_to_ndc(gt, creature.translation()) {
                if ndc.x < -0.4 {
                    transform.translation.x -= time.delta_seconds() * CAMERA_SPEED;
                } else if ndc.x > 0.4 {
                    transform.translation.x += time.delta_seconds() * CAMERA_SPEED;
                }
                if ndc.y < -0.8 {
                    transform.translation.y -= time.delta_seconds() * CAMERA_SPEED;
                } else if ndc.y > 0.8 {
                    transform.translation.y += time.delta_seconds() * CAMERA_SPEED;
                }
            }
        }
    }
}

pub fn plank(start: Vec2, end: Vec2) -> impl Bundle {
    let angle = if end.x < start.x {
        f32::atan2(start.y - end.y, start.x - end.x)
    } else {
        f32::atan2(end.y - start.y, end.x - start.x)
    };
    let offset = (Quat::from_rotation_z(angle) * Vec3::new(0.0, PLANK_THICKNESS * 0.5, 0.0)).xy();
    rectangle(
        start.midpoint(end) - offset,
        Vec2::new(start.distance(end), PLANK_THICKNESS),
        angle,
    )
}

pub fn wall(topleft: Vec2, bottomright: Vec2) -> impl Bundle {
    rectangle(
        topleft.midpoint(bottomright),
        (bottomright - topleft).abs(),
        0.0,
    )
}

pub fn rectangle(center: Vec2, size: Vec2, rotation: f32) -> impl Bundle {
    (
        SpriteBundle {
            sprite: Sprite {
                color: STATIC_COLOR,
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            transform: Transform::from_translation(center.extend(-1.0))
                .with_scale(size.extend(1.0))
                .with_rotation(Quat::from_rotation_z(rotation)),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(1.0, 1.0),
    )
}

#[derive(Component, Clone, Copy)]
pub struct Glass {}

pub fn spawn_glass(commands: &mut StateLocalSpawner<'_, '_>, bottom: Vec2, height: f32) {
    let sprite = Sprite {
        color: GLASS_COLOR,
        custom_size: Some(Vec2::ONE),
        ..default()
    };
    let size = Vec3::new(GLASS_THICKNESS, height / 3., 1.0);
    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                sprite: sprite.clone(),
                transform: Transform::from_translation(Vec3::new(
                    bottom.x,
                    bottom.y + height * (1 + 2 * i) as f32 / 6.,
                    0.0,
                ))
                .with_scale(size),
                ..default()
            },
            RigidBody::Static,
            Collider::rectangle(1.0, 1.0),
            ColliderDensity(0.5),
            Glass {},
        ));
    }
}

fn glass_collision(
    mut collision_event_reader: EventReader<CollisionStarted>,
    creatures: Query<(), With<Creature>>,
    mut glasses: Query<&mut RigidBody, With<Glass>>,
    joints: Query<&FixedJoint>,
    mut sounds: EventWriter<Sounds>,
) {
    'outer: for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if creatures.contains(*e1) {
            if let Ok(mut rb) = glasses.get_mut(*e2) {
                for joint in joints.iter() {
                    if joint.entity1 == *e1 || joint.entity2 == *e1 {
                        *rb = RigidBody::Dynamic;
                        sounds.send(Sounds::Glass);
                        continue 'outer;
                    }
                }
            }
        } else if creatures.contains(*e2) {
            if let Ok(mut rb) = glasses.get_mut(*e1) {
                for joint in joints.iter() {
                    if joint.entity1 == *e2 || joint.entity2 == *e2 {
                        *rb = RigidBody::Dynamic;
                        sounds.send(Sounds::Glass);
                        continue 'outer;
                    }
                }
            }
        }
    }
}

pub fn background(
    color: Color,
    topleft: Vec2,
    bottomright: Vec2,
    rotation: f32,
    z: f32,
) -> impl Bundle {
    SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::ONE),
            ..default()
        },
        transform: Transform::from_translation(topleft.midpoint(bottomright).extend(z))
            .with_scale((bottomright - topleft).abs().extend(1.0))
            .with_rotation(Quat::from_rotation_z(rotation)),
        ..default()
    }
}

#[derive(Component, Clone, Copy)]
pub struct Door(u16);

pub fn door(number: u16, bottom: Vec2, height: f32) -> impl Bundle {
    (
        SpriteBundle {
            sprite: Sprite {
                color: DOOR_COLOR,
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                bottom.x,
                bottom.y + height * 0.5,
                0.0,
            ))
            .with_scale(Vec3::new(DOOR_THICKNESS, height, 1.0)),
            ..default()
        },
        RigidBody::Kinematic,
        Collider::rectangle(1.0, 1.0),
        Door(number),
    )
}

#[derive(Component, Clone, Copy)]
pub struct PressurePlate(u16, Signal);

#[derive(Event, Debug, Clone, Copy)]
pub struct PressurePlateEvent(pub Entity, pub Signal, pub bool);

pub fn spawn_pressure_plate(
    commands: &mut StateLocalSpawner<'_, '_>,
    signal: Signal,
    center: Vec2,
    width: f32,
    rotation: f32,
) -> Entity {
    commands
        .spawn((
            TransformBundle {
                local: Transform::from_translation(Vec3::new(
                    center.x,
                    center.y + SENSOR_THICKNESS * 0.5,
                    0.0,
                ))
                .with_scale(Vec3::new(width, SENSOR_THICKNESS, 1.0))
                .with_rotation(Quat::from_rotation_z(rotation)),
                ..default()
            },
            VisibilityBundle::default(),
            RigidBody::Static,
            Collider::rectangle(1.0, 1.0),
            Sensor,
            PressurePlate(0, signal),
        ))
        .with_children(|cb| {
            cb.spawn((SpriteBundle {
                sprite: Sprite {
                    color: SENSOR_COLOR,
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                transform: Transform::IDENTITY,
                ..default()
            },));
        })
        .id()
}

pub fn spawn_exit(
    commands: &mut StateLocalSpawner<'_, '_>,
    center: Vec2,
    width: f32,
    rotation: f32,
) {
    let e = spawn_pressure_plate(commands, Signal::NextLevel, center, width, rotation);
    commands.entity(e).with_children(|cb| {
        cb.spawn((SpriteBundle {
            sprite: Sprite {
                color: EXIT_COLOR,
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -0.5 + EXIT_HEIGHT / SENSOR_THICKNESS * 0.5, -0.1)
                .with_scale(Vec3::new(0.9, EXIT_HEIGHT / SENSOR_THICKNESS, 1.0)),
            ..default()
        },));
    });
}

fn on_pressure_enter(
    mut collision_event_reader: EventReader<CollisionStarted>,
    creatures: Query<(), With<Creature>>,
    mut plates: Query<&mut PressurePlate>,
    mut event: EventWriter<PressurePlateEvent>,
) {
    for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if creatures.contains(*e1) {
            if let Ok(mut plate) = plates.get_mut(*e2) {
                plate.0 += 1;
                if plate.0 == 1 {
                    event.send(PressurePlateEvent(*e2, plate.1, true));
                }
            }
        } else if creatures.contains(*e2) {
            if let Ok(mut plate) = plates.get_mut(*e1) {
                plate.0 += 1;
                if plate.0 == 1 {
                    event.send(PressurePlateEvent(*e1, plate.1, true));
                }
            }
        }
    }
}

fn on_pressure_exit(
    mut collision_event_reader: EventReader<CollisionEnded>,
    creatures: Query<(), With<Creature>>,
    mut plates: Query<&mut PressurePlate>,
    mut event: EventWriter<PressurePlateEvent>,
) {
    for CollisionEnded(e1, e2) in collision_event_reader.read() {
        if creatures.contains(*e1) {
            if let Ok(mut plate) = plates.get_mut(*e2) {
                plate.0 -= 1;
                if plate.0 == 0 {
                    event.send(PressurePlateEvent(*e2, plate.1, false));
                }
            }
        } else if creatures.contains(*e2) {
            if let Ok(mut plate) = plates.get_mut(*e1) {
                plate.0 -= 1;
                if plate.0 == 0 {
                    event.send(PressurePlateEvent(*e1, plate.1, false));
                }
            }
        }
    }
}

fn on_pressure_event(
    mut commands: Commands,
    mut event: EventReader<PressurePlateEvent>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
    mut doors: Query<(Entity, &Door, &mut Visibility)>,
    mut sounds: EventWriter<Sounds>,
) {
    for PressurePlateEvent(entity, signal, pressed) in event.read() {
        for child in children.iter_descendants(*entity) {
            if let Ok(mut t) = transforms.get_mut(child) {
                t.translation.y = if *pressed { -0.75 } else { 0.0 };
            }
            if let Signal::Door(i) = signal {
                for (e, door, mut v) in doors.iter_mut() {
                    if door.0 == *i {
                        if *pressed {
                            commands.entity(e).insert(Sensor {});
                            *v = Visibility::Hidden;
                        } else {
                            // commands.entity(e).remove::<Sensor>();
                            // *v = Visibility::Inherited;
                        }
                    }
                }
            }
        }
        sounds.send(Sounds::Click);
    }
}
