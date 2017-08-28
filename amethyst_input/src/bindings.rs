//! Defines binding structure used for saving and loading input settings.

use smallvec::SmallVec;
use fnv::FnvHashMap as HashMap;
use super::{Axis, Button};

/// Used for saving and loading input settings.
#[derive(Default, Serialize, Deserialize)]
pub struct Bindings {
    pub(super) axes: HashMap<String, Axis>,
    pub(super) actions: HashMap<String, SmallVec<[Button; 4]>>,
}

impl Bindings {
    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<T: Into<String>>(&mut self, id: T, axis: Axis) -> Option<Axis> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<T: AsRef<str>>(&mut self, id: T) -> Option<Axis> {
        self.axes.remove(id.as_ref())
    }

    /// Returns a reference to an axis.
    pub fn axis<T: AsRef<str>>(&mut self, id: T) -> Option<&Axis> {
        self.axes.get(id.as_ref())
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> Vec<String> {
        self.axes
            .keys()
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Add a button to an action.
    ///
    /// This will insert a new binding between this action and the button.
    pub fn insert_action_binding<T: AsRef<str>>(&mut self, id: T, binding: Button) {
        let mut make_new = false;
        match self.actions.get_mut(id.as_ref()) {
            Some(action_bindings) => {
                if action_bindings.iter().all(|&b| b != binding) {
                    action_bindings.push(binding);
                }
            }
            None => {
                make_new = true;
            }
        }
        if make_new {
            let mut bindings = SmallVec::new();
            bindings.push(binding);
            self.actions.insert(id.as_ref().to_string(), bindings);
        }
    }

    /// Removes an action binding that was assigned previously.
    pub fn remove_action_binding<T: AsRef<str>>(&mut self, id: T, binding: Button) {
        let mut kill_it = false;
        if let Some(action_bindings) = self.actions.get_mut(id.as_ref()) {
            let index = action_bindings.iter().position(|&b| b == binding);
            if let Some(index) = index {
                action_bindings.swap_remove(index);
            }
            kill_it = action_bindings.is_empty();
        }
        if kill_it {
            self.actions.remove(id.as_ref());
        }
    }

    /// Returns an action's bindings.
    pub fn action_bindings<T: AsRef<str>>(&self, id: T) -> Option<&[Button]> {
        self.actions.get(id.as_ref()).map(|a| &**a)
    }

    /// Gets a list of all action bindings
    pub fn actions(&self) -> Vec<String> {
        self.actions
            .keys()
            .cloned()
            .collect::<Vec<String>>()
    }
}
