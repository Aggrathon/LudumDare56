use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use enum_iterator::{all, Sequence};

#[derive(Default)]
pub struct IdentityTransitionsPlugin<S: States>(PhantomData<S>);

impl<S: States> Plugin for IdentityTransitionsPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            StateTransition,
            last_transition::<S>
                .pipe(run_reenter::<S>)
                .in_set(EnterSchedules::<S>::default()),
        )
        .add_systems(
            StateTransition,
            last_transition::<S>
                .pipe(run_reexit::<S>)
                .in_set(ExitSchedules::<S>::default()),
        );
    }
}

fn run_reenter<S: States>(transition: In<Option<StateTransitionEvent<S>>>, world: &mut World) {
    let Some(transition) = transition.0 else {
        return;
    };
    if transition.entered != transition.exited {
        return;
    }
    let Some(entered) = transition.entered else {
        return;
    };
    let _ = world.try_run_schedule(OnEnter(entered));
}

fn run_reexit<S: States>(transition: In<Option<StateTransitionEvent<S>>>, world: &mut World) {
    let Some(transition) = transition.0 else {
        return;
    };
    if transition.entered != transition.exited {
        return;
    }
    let Some(exited) = transition.exited else {
        return;
    };
    let _ = world.try_run_schedule(OnExit(exited));
}

#[derive(Component, Clone)]
pub struct StateLocal {}

pub fn despawn_state_local(mut commands: Commands, query: Query<Entity, With<StateLocal>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct StateLocalSpawner<'w, 's>(pub Commands<'w, 's>);

impl<'w, 's> StateLocalSpawner<'w, 's> {
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.0.spawn((StateLocal {}, bundle))
    }
}

#[derive(Default)]
pub struct StateLocalPlugin<S: States + Sequence>(PhantomData<S>);

impl<S: States + Sequence> Plugin for StateLocalPlugin<S> {
    fn build(&self, app: &mut App) {
        for state in all::<S>() {
            app.add_systems(OnExit(state), despawn_state_local);
        }
    }
}
