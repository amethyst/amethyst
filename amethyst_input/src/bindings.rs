//! Defines binding structure used for saving and loading input settings.

use super::{Axis, Button};
use fnv::FnvHashMap as HashMap;
use smallvec::SmallVec;
use std::borrow::Borrow;
use std::hash::Hash;

/// Used for saving and loading input settings.
#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Default(bound = ""))]
pub struct Bindings<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    pub(super) axes: HashMap<AX, Axis>,
    pub(super) actions: HashMap<AC, SmallVec<[Button; 4]>>,
}

impl<AX, AC> Bindings<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    /// Creates a new empty Bindings structure
    pub fn new() -> Self {
        Default::default()
    }
}

impl<AX, AC> Bindings<AX, AC>
where
    AX: Hash + Eq + Clone,
    AC: Hash + Eq + Clone,
{
    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<A: Into<AX>>(&mut self, id: A, axis: Axis) -> Option<Axis> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<A: Hash + Eq + ?Sized>(&mut self, id: &A) -> Option<Axis>
    where
        AX: Borrow<A>,
    {
        self.axes.remove(id)
    }

    /// Returns a reference to an axis.
    pub fn axis<A: Hash + Eq + ?Sized>(&mut self, id: &A) -> Option<&Axis>
    where
        AX: Borrow<A>,
    {
        self.axes.get(id)
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> Vec<AX> {
        self.axes.keys().cloned().collect::<Vec<AX>>()
    }

    /// Add a button to an action.
    ///
    /// This will insert a new binding between this action and the button.
    pub fn insert_action_binding<A>(&mut self, id: A, binding: Button)
    where
        A: Hash + Eq + Into<AC>,
        AC: Borrow<A>,
    {
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
            self.actions.insert(id.into(), bindings);
        }
    }

    /// Removes an action binding that was assigned previously.
    pub fn remove_action_binding<T: Hash + Eq + ?Sized>(&mut self, id: &T, binding: Button)
    where
        AC: Borrow<T>,
    {
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
    pub fn action_bindings<T: Hash + Eq + ?Sized>(&self, id: &T) -> Option<&[Button]>
    where
        AC: Borrow<T>,
    {
        self.actions.get(id).map(|a| &**a)
    }

    /// Gets a list of all action bindings
    pub fn actions(&self) -> Vec<AC> {
        self.actions.keys().cloned().collect::<Vec<AC>>()
    }
}
