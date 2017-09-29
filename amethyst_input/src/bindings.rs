//! Defines binding structure used for saving and loading input settings.

use std::hash::Hash;

use fnv::FnvHashMap as HashMap;
use smallvec::SmallVec;

use super::{Axis, Button};

/// Used for saving and loading input settings.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Bindings<T> where T : Hash + Eq {
    pub(super) axes: HashMap<String, Axis>,
    pub(super) actions: HashMap<T, SmallVec<[Button; 4]>>,
}

impl<T> Bindings<T> where T: Hash + Eq + Clone {
    /// Creates a new empty Bindings structure
    pub fn new() -> Self {
        Self {
            axes: HashMap::default(),
            actions: HashMap::default(),
        }
    }

    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<A: Into<String>>(&mut self, id: A, axis: Axis) -> Option<Axis> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<A: AsRef<str>>(&mut self, id: A) -> Option<Axis> {
        self.axes.remove(id.as_ref())
    }

    /// Returns a reference to an axis.
    pub fn axis<A: AsRef<str>>(&mut self, id: A) -> Option<&Axis> {
        self.axes.get(id.as_ref())
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> Vec<String> {
        self.axes.keys().cloned().collect::<Vec<String>>()
    }

    /// Add a button to an action.
    ///
    /// This will insert a new binding between this action and the button.
    pub fn insert_action_binding(&mut self, id: T, binding: Button) {
        let mut make_new = false;
        match self.actions.get_mut(&id) {
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
            self.actions.insert(id, bindings);
        }
    }

    /// Removes an action binding that was assigned previously.
    pub fn remove_action_binding(&mut self, id: &T, binding: Button) {
        let mut kill_it = false;
        if let Some(action_bindings) = self.actions.get_mut(id) {
            let index = action_bindings.iter().position(|&b| b == binding);
            if let Some(index) = index {
                action_bindings.swap_remove(index);
            }
            kill_it = action_bindings.is_empty();
        }
        if kill_it {
            self.actions.remove(id);
        }
    }

    /// Returns an action's bindings.
    pub fn action_bindings(&self, id: &T) -> Option<&[Button]> {
        self.actions.get(id).map(|a| &**a)
    }

    /// Gets a list of all action bindings
    pub fn actions(&self) -> Vec<T> {
        self.actions.keys().cloned().collect::<Vec<T>>()
    }
}
