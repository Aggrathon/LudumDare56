use avian2d::prelude::*;
use bevy::prelude::*;

const PLANK_THICKNESS: f32 = 25.0;
const STATIC_COLOR: Color = Color::srgb(0.8, 0.75, 1.0);

pub struct ObjectPlugin;

impl Plugin for ObjectPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
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
