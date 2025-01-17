use bevy::{app::PluginGroupBuilder, prelude::*};
use camera::CameraPlugin;
pub use camera::MoveFocusEvent;
use distance::DistancePlugin;
pub use distance::{CameraDistance, DistanceLabels};

mod camera;
mod distance;

pub struct CameraPluginGroup;

impl PluginGroup for CameraPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CameraPlugin).add(DistancePlugin);
    }
}
