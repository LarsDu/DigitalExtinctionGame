//! This crate implements handling of user input.

use bevy::{app::PluginGroupBuilder, prelude::*};
use command::CommandPlugin;
use pointer::PointerPlugin;
use selection::SelectionPlugin;

mod command;
mod pointer;
mod selection;

pub struct ControllerPluginGroup;

impl PluginGroup for ControllerPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(PointerPlugin)
            .add(CommandPlugin)
            .add(SelectionPlugin);
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum Labels {
    PreInputUpdate,
    InputUpdate,
}