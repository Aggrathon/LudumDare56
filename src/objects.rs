use avian2d::prelude::*;
use bevy::prelude::*;

use crate::creature::Creature;
use crate::utils::StateLocalSpawner;

const PLANK_THICKNESS: f32 = 25.0;
const SENSOR_THICKNESS: f32 = 6.0;

const STATIC_COLOR: Color = Color::srgb(0.8, 0.75, 1.0);
const SENSOR_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (on_pressure_enter, on_pressure_exit, on_pressure_event),
        )
        .add_event::<PressurePlateEvent>();
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
pub struct PressurePlate(u32);

#[derive(Event, Debug, Clone, Copy)]
struct PressurePlateEvent(Entity, bool);

pub fn spawn_pressure_plate(
    commands: &mut StateLocalSpawner<'_, '_>,
    center: Vec2,
    width: f32,
    rotation: f32,
) {
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
            PressurePlate(0),
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
                    event.send(PressurePlateEvent(*e2, true));
                }
            }
        } else if creatures.contains(*e2) {
            if let Ok(mut plate) = plates.get_mut(*e1) {
                plate.0 += 1;
                if plate.0 == 1 {
                    event.send(PressurePlateEvent(*e1, true));
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
                    event.send(PressurePlateEvent(*e2, false));
                }
            }
        } else if creatures.contains(*e2) {
            if let Ok(mut plate) = plates.get_mut(*e1) {
                plate.0 -= 1;
                if plate.0 == 0 {
                    event.send(PressurePlateEvent(*e1, false));
                }
            }
        }
    }
}

fn on_pressure_event(
    mut event: EventReader<PressurePlateEvent>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
) {
    for PressurePlateEvent(entity, pressed) in event.read() {
        for child in children.iter_descendants(*entity) {
            if let Ok(mut t) = transforms.get_mut(child) {
                t.translation.y = if *pressed { -0.75 } else { 0.0 };
            }
        }
        // TODO click sound
    }
}
