use gtk::{CheckButton, ToggleButtonExt};
use crate::config_management;

pub struct SignalHandle {}
impl SignalHandle {
    pub fn follow_mouse_switch_changed(self, switch: CheckButton) {
        config_management::set_bool("default", "mousecheck", switch.get_active());
    }
}