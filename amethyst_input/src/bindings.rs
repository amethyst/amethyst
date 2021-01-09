//! Defines binding structure used for saving and loading input settings.

use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
};

use fnv::FnvHashMap as HashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::{axis, Axis, Button};

/// Used for saving and loading input settings.
///
/// An action can either be a single button or a combination of them.
///
/// # Examples
///
/// Example Ron config file:
/// ```ron
/// (
///     axes: {
///         "updown": Emulated(
///             pos: Key(Up),
///             neg: Key(Down)
///         ),
///         "leftright": Multiple([ // Multiple bindings for one axis
///             Emulated(
///                 pos: Key(Right),
///                 neg: Key(Left)
///             ),
///             Emulated(
///                 pos: Key(D),
///                 neg: Key(A)
///             )
///         ])
///     },
///     actions: {
///         "fire": [ [Mouse(Left)], [Key(X)] ], // Multiple bindings for one action
///         "reload": [ [Key(LControl), Key(R)] ] // Combinations of multiple bindings possible
///     }
/// )
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Bindings {
    pub(super) axes: HashMap<Cow<'static, str>, Axis>,
    /// The inner array here is for button combinations, the other is for different possibilities.
    ///
    /// So for example if you want to quit by either "Esc" or "Ctrl+q" you would have
    /// `[[Esc], [Ctrl, Q]]`.
    pub(super) actions: HashMap<Cow<'static, str>, SmallVec<[SmallVec<[Button; 2]>; 4]>>,
}

/// An enum of possible errors that can occur when binding an action or axis.
#[derive(Clone, Debug, PartialEq)]
pub enum BindingError {
    /// Axis buttons have overlap with an action combo of length one.
    AxisButtonAlreadyBoundToAction(Cow<'static, str>, Button),
    /// Axis buttons provided have overlap with an existing axis.
    AxisButtonAlreadyBoundToAxis(Cow<'static, str>, Axis),
    /// A combo of length one was provided, and it overlaps with an axis binding.
    ButtonBoundToAxis(Cow<'static, str>, Axis),
    /// Combo provided was already bound to the contained action.
    ComboAlreadyBound(Cow<'static, str>),
    /// Combo provided for action binding has two (or more) of the same button.
    ComboContainsDuplicates(Cow<'static, str>),
    /// That specific axis on that specific controller is already in use for an
    /// axis binding.
    ControllerAxisAlreadyBound(Cow<'static, str>),
    /// The given axis was already bound for use
    MouseAxisAlreadyBound(Cow<'static, str>),
    /// You attempted to bind a mousewheel axis twice.
    MouseWheelAxisAlreadyBound(Cow<'static, str>),
}

impl Display for BindingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            BindingError::ComboContainsDuplicates(ref id) => {
                write!(
                    f,
                    "Combo provided contained two (or more) of the same button: {}",
                    id
                )
            }
            BindingError::ComboAlreadyBound(ref action) => {
                write!(f, "Combo provided was already bound to action {}", action)
            }
            BindingError::ButtonBoundToAxis(ref id, ref _axis) => {
                write!(f, "Button provided was a button in use by axis {}", id)
            }
            BindingError::AxisButtonAlreadyBoundToAxis(ref id, ref _axis) => {
                write!(
                    f,
                    "Axis provided contained a button that's already in use by axis {}",
                    id
                )
            }
            BindingError::AxisButtonAlreadyBoundToAction(ref id, ref _action) => {
                write!(
                f,
                "Axis provided contained a button that's already in use by single button action {}",
                id
            )
            }
            BindingError::ControllerAxisAlreadyBound(ref id) => {
                write!(f, "Controller axis provided is already in use by {}", id)
            }
            BindingError::MouseAxisAlreadyBound(ref id) => {
                write!(f, "Mouse axis provided is already in use by {}", id)
            }
            BindingError::MouseWheelAxisAlreadyBound(ref id) => {
                write!(f, "Mouse wheel axis provided is already in use by {}", id)
            }
        }
    }
}

impl Error for BindingError {}

/// An enum of possible errors that can occur when removing an action binding.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionRemovedError {
    /// The action has bindings, but this isn't one of them.
    ActionExistsButBindingDoesnt,
    /// The action provided doesn't exist.
    ActionNotFound,
    /// Combo provided for action binding has two (or more) of the same button.
    BindingContainsDuplicates,
}

impl Display for ActionRemovedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            ActionRemovedError::ActionExistsButBindingDoesnt => {
                write!(f, "Action found, but binding isn't present")
            }
            ActionRemovedError::ActionNotFound => write!(f, "Action not found"),
            ActionRemovedError::BindingContainsDuplicates => {
                write!(f, "Binding removal requested contains duplicate buttons")
            }
        }
    }
}

impl Error for ActionRemovedError {}

impl Bindings {
    /// Creates a new empty Bindings structure
    pub fn new() -> Self {
        Default::default()
    }
}

impl Bindings {
    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<A: Into<Cow<'static, str>>>(
        &mut self,
        id: A,
        axis: Axis,
    ) -> Result<Option<Axis>, BindingError> {
        let id = id.into();
        self.check_axis_invariants(&id, &axis)?;
        Ok(self.axes.insert(id, axis))
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<A>(&mut self, id: &A) -> Option<Axis>
    where
        Cow<'static, str>: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.axes.remove(id)
    }

    /// Returns a reference to an axis.
    pub fn axis<A>(&self, id: &A) -> Option<&Axis>
    where
        Cow<'static, str>: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.axes.get(id)
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> impl Iterator<Item = &Cow<'static, str>> {
        self.axes.keys()
    }

    /// Add a button or button combination to an action.
    ///
    /// This will attempt to insert a new binding between this action and the button(s).
    pub fn insert_action_binding<B: IntoIterator<Item = Button>>(
        &mut self,
        id: Cow<'static, str>,
        binding: B,
    ) -> Result<(), BindingError> {
        let bind: SmallVec<[Button; 2]> = binding.into_iter().collect();
        self.check_action_invariants(&id, bind.as_slice())?;
        let mut make_new = false;
        match self.actions.get_mut(&id) {
            Some(action_bindings) => {
                action_bindings.push(bind.clone());
            }
            None => {
                make_new = true;
            }
        }
        if make_new {
            let mut bindings = SmallVec::new();
            bindings.push(bind);
            self.actions.insert(id, bindings);
        }
        Ok(())
    }

    /// Removes an action binding that was assigned previously.
    pub fn remove_action_binding<A>(
        &mut self,
        id: &A,
        binding: &[Button],
    ) -> Result<(), ActionRemovedError>
    where
        Cow<'static, str>: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        for i in 0..binding.len() {
            for j in (i + 1)..binding.len() {
                if binding[i] == binding[j] {
                    return Err(ActionRemovedError::BindingContainsDuplicates);
                }
            }
        }
        let kill_it;
        if let Some(action_bindings) = self.actions.get_mut(id) {
            let index = action_bindings.iter().position(|b| {
                b.len() == binding.len()
                    // The bindings can be provided in any order, but they must all
                    // be the same bindings.
                    && b.iter().all(|b| binding.iter().any(|binding| b == binding))
            });
            if let Some(index) = index {
                action_bindings.swap_remove(index);
            } else {
                return Err(ActionRemovedError::ActionExistsButBindingDoesnt);
            }
            kill_it = action_bindings.is_empty();
        } else {
            return Err(ActionRemovedError::ActionNotFound);
        }
        if kill_it {
            self.actions.remove(id);
        }
        Ok(())
    }

    /// Returns an action's bindings.
    pub fn action_bindings<A>(&self, id: &A) -> impl Iterator<Item = &[Button]>
    where
        Cow<'static, str>: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.actions
            .get(id)
            .map(SmallVec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(SmallVec::as_slice)
    }

    /// Gets a list of all action bindings
    pub fn actions(&self) -> impl Iterator<Item = &Cow<'static, str>> {
        self.actions.keys()
    }

    /// Check that this structure upholds its guarantees. Should only be necessary when serializing or deserializing the bindings.
    pub fn check_invariants(&mut self) -> Result<(), BindingError> {
        // The easiest way to do this is to use the existing code that checks for invariants when adding bindings.
        // So we'll just remove and then re-add all of the bindings.

        let action_bindings = self
            .actions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        for (k, v) in action_bindings {
            for c in v {
                self.remove_action_binding(&k, &c)
                    .expect("Unreachable: We just cloned the bindings, they can't be incorrect.");
                self.insert_action_binding(k.clone(), c)?;
            }
        }
        let axis_bindings = self
            .axes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        for (k, a) in axis_bindings {
            self.remove_axis(&k);
            self.insert_axis(k, a)?;
        }
        Ok(())
    }

    fn check_action_invariants(&self, id: &str, bind: &[Button]) -> Result<(), BindingError> {
        // Guarantee each button is unique.
        for i in 0..bind.len() {
            for j in (i + 1)..bind.len() {
                if bind[i] == bind[j] {
                    return Err(BindingError::ComboContainsDuplicates(id.to_owned().into()));
                }
            }
        }
        if bind.len() == 1 {
            for (k, a) in self.axes.iter() {
                if a.conflicts_with_button(&bind[0]) {
                    return Err(BindingError::ButtonBoundToAxis(k.clone(), a.clone()));
                }
            }
        }
        for (k, a) in self.actions.iter() {
            for c in a {
                if c.len() == bind.len() && bind.iter().all(|bind| c.iter().any(|c| c == bind)) {
                    return Err(BindingError::ComboAlreadyBound(k.clone()));
                }
            }
        }
        Ok(())
    }

    fn check_axis_invariants(&self, id: &str, axis: &Axis) -> Result<(), BindingError> {
        for (k, a) in self.axes.iter().filter(|(k, _a)| *k != id) {
            if let Some(conflict_type) = axis.conflicts_with_axis(a) {
                return Err(match conflict_type {
                    axis::Conflict::Button => {
                        BindingError::AxisButtonAlreadyBoundToAxis(k.clone(), a.clone())
                    }
                    axis::Conflict::ControllerAxis => {
                        BindingError::ControllerAxisAlreadyBound(k.clone())
                    }
                    axis::Conflict::MouseAxis => BindingError::MouseAxisAlreadyBound(k.clone()),
                    axis::Conflict::MouseWheelAxis => {
                        BindingError::MouseWheelAxisAlreadyBound(k.clone())
                    }
                });
            }
        }

        for (k, a) in self.actions.iter() {
            for c in a {
                // Since you can't bind combos to an axis we only need to check combos with length 1.
                if c.len() == 1 && axis.conflicts_with_button(&c[0]) {
                    return Err(BindingError::AxisButtonAlreadyBoundToAction(
                        k.clone(),
                        c[0],
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use winit::event::{MouseButton, VirtualKeyCode};

    use super::*;
    use crate::{button::*, controller::ControllerAxis};

    #[test]
    fn add_and_remove_actions() {
        const TEST_ACTION: Cow<'static, str> = Cow::Borrowed("test_action");

        let mut bindings = Bindings::new();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings(&TEST_ACTION).next(), None);

        bindings
            .insert_action_binding(
                TEST_ACTION,
                [Button::Mouse(MouseButton::Left)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(bindings.actions().collect::<Vec<_>>(), vec![&TEST_ACTION]);
        let action_bindings = bindings.action_bindings(&TEST_ACTION).collect::<Vec<_>>();
        assert_eq!(action_bindings, vec![[Button::Mouse(MouseButton::Left)]]);
        bindings
            .remove_action_binding(&TEST_ACTION, &[Button::Mouse(MouseButton::Left)])
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings(&TEST_ACTION).next(), None);

        bindings
            .insert_action_binding(
                TEST_ACTION,
                [
                    Button::Mouse(MouseButton::Left),
                    Button::Mouse(MouseButton::Right),
                ]
                .iter()
                .cloned(),
            )
            .unwrap();
        assert_eq!(bindings.actions().collect::<Vec<_>>(), vec![&TEST_ACTION],);
        let action_bindings = bindings.action_bindings(&TEST_ACTION).collect::<Vec<_>>();
        assert_eq!(
            action_bindings,
            vec![[
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right)
            ]]
        );
        bindings
            .remove_action_binding(
                &TEST_ACTION,
                &[
                    Button::Mouse(MouseButton::Left),
                    Button::Mouse(MouseButton::Right),
                ],
            )
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings(&TEST_ACTION).next(), None);

        bindings
            .insert_action_binding(
                TEST_ACTION,
                [
                    Button::Mouse(MouseButton::Left),
                    Button::Mouse(MouseButton::Right),
                ]
                .iter()
                .cloned(),
            )
            .unwrap();
        assert_eq!(
            bindings
                .remove_action_binding(
                    &TEST_ACTION,
                    &[
                        Button::Mouse(MouseButton::Right),
                        Button::Mouse(MouseButton::Right),
                    ],
                )
                .unwrap_err(),
            ActionRemovedError::BindingContainsDuplicates
        );
        assert_eq!(
            bindings
                .remove_action_binding(&TEST_ACTION, &[Button::Mouse(MouseButton::Left),],)
                .unwrap_err(),
            ActionRemovedError::ActionExistsButBindingDoesnt
        );
        assert_eq!(
            bindings
                .remove_action_binding("nonsense_action", &[Button::Mouse(MouseButton::Left),],)
                .unwrap_err(),
            ActionRemovedError::ActionNotFound
        );
        let actions = bindings.actions().collect::<Vec<_>>();
        assert_eq!(actions, vec![&TEST_ACTION]);
        let action_bindings = bindings.action_bindings(&TEST_ACTION).collect::<Vec<_>>();
        assert_eq!(
            action_bindings,
            vec![[
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right)
            ]]
        );
        bindings
            .remove_action_binding(
                &TEST_ACTION,
                &[
                    Button::Mouse(MouseButton::Right),
                    Button::Mouse(MouseButton::Left),
                ],
            )
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings(&TEST_ACTION).next(), None);
    }
    #[test]
    fn insert_errors() {
        const TEST_ACTION: Cow<'static, str> = Cow::Borrowed("test_action");
        const TEST_ACTION_2: Cow<'static, str> = Cow::Borrowed("test_action_2");
        const TEST_AXIS: Cow<'static, str> = Cow::Borrowed("test_axis");
        const TEST_AXIS_2: Cow<'static, str> = Cow::Borrowed("test_axis_2");
        const TEST_CONTROLLER_AXIS: Cow<'static, str> = Cow::Borrowed("test_controller_axis");
        const TEST_CONTROLLER_AXIS_2: Cow<'static, str> = Cow::Borrowed("test_controller_axis_2");
        const TEST_MOUSEWHEEL_AXIS: Cow<'static, str> = Cow::Borrowed("test_mousewheel_axis");
        const TEST_MOUSEWHEEL_AXIS_2: Cow<'static, str> = Cow::Borrowed("test_mousewheel_axis_2");
        let mut bindings = Bindings::new();
        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_ACTION,
                    [
                        Button::Mouse(MouseButton::Left),
                        Button::Mouse(MouseButton::Right),
                        Button::Mouse(MouseButton::Left),
                    ]
                    .iter()
                    .cloned(),
                )
                .unwrap_err(),
            BindingError::ComboContainsDuplicates(TEST_ACTION)
        );
        bindings
            .insert_action_binding(
                TEST_ACTION,
                [Button::Mouse(MouseButton::Left)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_ACTION,
                    [Button::Mouse(MouseButton::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ComboAlreadyBound(TEST_ACTION)
        );
        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_ACTION_2,
                    [Button::Mouse(MouseButton::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ComboAlreadyBound(TEST_ACTION)
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_AXIS,
                    Axis::Emulated {
                        pos: Button::Mouse(MouseButton::Left),
                        neg: Button::Mouse(MouseButton::Right),
                    },
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAction(
                TEST_ACTION,
                Button::Mouse(MouseButton::Left)
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_AXIS,
                    Axis::Multiple(vec![Axis::Emulated {
                        pos: Button::Mouse(MouseButton::Left),
                        neg: Button::Mouse(MouseButton::Right),
                    }])
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAction(
                TEST_ACTION,
                Button::Mouse(MouseButton::Left)
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_AXIS,
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::Left),
                        neg: Button::Key(VirtualKeyCode::Right),
                    },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_ACTION_2,
                    [Button::Key(VirtualKeyCode::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ButtonBoundToAxis(
                TEST_AXIS,
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_AXIS_2,
                    [Button::Key(VirtualKeyCode::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ButtonBoundToAxis(
                TEST_AXIS,
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_AXIS_2,
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::Left),
                        neg: Button::Key(VirtualKeyCode::Up),
                    },
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAxis(
                TEST_AXIS,
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_CONTROLLER_AXIS,
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::RightX,
                        invert: false,
                        dead_zone: 0.25,
                    },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_CONTROLLER_AXIS,
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::LeftX,
                        invert: false,
                        dead_zone: 0.25,
                    },
                )
                .unwrap(),
            Some(Axis::Controller {
                controller_id: 0,
                axis: ControllerAxis::RightX,
                invert: false,
                dead_zone: 0.25,
            })
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_CONTROLLER_AXIS_2,
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::LeftX,
                        invert: true,
                        dead_zone: 0.1,
                    },
                )
                .unwrap_err(),
            BindingError::ControllerAxisAlreadyBound(TEST_CONTROLLER_AXIS)
        );
        assert_eq!(
            bindings
                .insert_axis(TEST_MOUSEWHEEL_AXIS, Axis::MouseWheel { horizontal: true },)
                .unwrap(),
            None
        );
        assert_eq!(
            bindings
                .insert_axis(TEST_MOUSEWHEEL_AXIS, Axis::MouseWheel { horizontal: false },)
                .unwrap(),
            Some(Axis::MouseWheel { horizontal: true })
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_MOUSEWHEEL_AXIS_2,
                    Axis::MouseWheel { horizontal: false },
                )
                .unwrap_err(),
            BindingError::MouseWheelAxisAlreadyBound(TEST_MOUSEWHEEL_AXIS)
        );
    }

    #[test]
    fn multiple_axis_conflicts() {
        const TEST_ACTION: Cow<'static, str> = Cow::Borrowed("test_action");
        const NORMAL_AXIS: Cow<'static, str> = Cow::Borrowed("normal_axis");
        const MULTIPLE_AXIS: Cow<'static, str> = Cow::Borrowed("multiple_axis");
        const NORMAL_CONTROLLER_AXIS: Cow<'static, str> = Cow::Borrowed("normal_controller_axis");
        const MULTIPLE_CONTROLLER_AXIS: Cow<'static, str> =
            Cow::Borrowed("multiple_controller_axis");
        const MULTIPLE_CONTROLLER_AXIS_2: Cow<'static, str> =
            Cow::Borrowed("multiple_controller_axis_2");

        let mut bindings = Bindings::new();
        assert_eq!(
            bindings
                .insert_axis(
                    NORMAL_AXIS,
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::A),
                        neg: Button::Key(VirtualKeyCode::B)
                    },
                )
                .unwrap(),
            None
        );

        assert_eq!(
            bindings
                .insert_axis(
                    MULTIPLE_AXIS,
                    Axis::Multiple(vec![
                        Axis::Emulated {
                            pos: Button::Key(VirtualKeyCode::C),
                            neg: Button::Key(VirtualKeyCode::D)
                        },
                        Axis::Emulated {
                            pos: Button::Key(VirtualKeyCode::A),
                            neg: Button::Key(VirtualKeyCode::E)
                        }
                    ])
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAxis(
                NORMAL_AXIS,
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::A),
                    neg: Button::Key(VirtualKeyCode::B)
                }
            )
        );

        assert_eq!(
            bindings
                .insert_axis(
                    MULTIPLE_AXIS,
                    Axis::Multiple(vec![
                        Axis::Emulated {
                            pos: Button::Key(VirtualKeyCode::C),
                            neg: Button::Key(VirtualKeyCode::D)
                        },
                        Axis::Emulated {
                            pos: Button::Key(VirtualKeyCode::E),
                            neg: Button::Key(VirtualKeyCode::F)
                        }
                    ])
                )
                .unwrap(),
            None
        );

        assert_eq!(
            bindings
                .insert_action_binding(
                    TEST_ACTION,
                    [Button::Key(VirtualKeyCode::C),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ButtonBoundToAxis(
                MULTIPLE_AXIS,
                Axis::Multiple(vec![
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::C),
                        neg: Button::Key(VirtualKeyCode::D)
                    },
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::E),
                        neg: Button::Key(VirtualKeyCode::F)
                    }
                ])
            )
        );

        assert_eq!(
            bindings
                .insert_axis(
                    NORMAL_CONTROLLER_AXIS,
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::RightX,
                        invert: false,
                        dead_zone: 0.25,
                    }
                )
                .unwrap(),
            None
        );

        assert_eq!(
            bindings
                .insert_axis(
                    MULTIPLE_CONTROLLER_AXIS,
                    Axis::Multiple(vec![Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::RightX,
                        invert: false,
                        dead_zone: 0.25,
                    }])
                )
                .unwrap_err(),
            BindingError::ControllerAxisAlreadyBound(NORMAL_CONTROLLER_AXIS)
        );

        assert_eq!(
            bindings
                .insert_axis(
                    MULTIPLE_CONTROLLER_AXIS,
                    Axis::Multiple(vec![Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::LeftX,
                        invert: false,
                        dead_zone: 0.25,
                    }])
                )
                .unwrap(),
            None
        );

        assert_eq!(
            bindings
                .insert_axis(
                    MULTIPLE_CONTROLLER_AXIS_2,
                    Axis::Multiple(vec![Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::LeftX,
                        invert: false,
                        dead_zone: 0.25,
                    }])
                )
                .unwrap_err(),
            BindingError::ControllerAxisAlreadyBound(MULTIPLE_CONTROLLER_AXIS)
        );
    }

    #[test]
    fn add_and_remove_axes() {
        const TEST_AXIS: Cow<'static, str> = Cow::Borrowed("test_axis");
        const TEST_CONTROLLER_AXIS: Cow<'static, str> = Cow::Borrowed("test_controller_axis");
        const TEST_MOUSEWHEEL_AXIS: Cow<'static, str> = Cow::Borrowed("test_mousewheel_axis");

        let mut bindings = Bindings::new();
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_AXIS,
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::Left),
                        neg: Button::Key(VirtualKeyCode::Right),
                    },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings.remove_axis("test_axis"),
            Some(Axis::Emulated {
                pos: Button::Key(VirtualKeyCode::Left),
                neg: Button::Key(VirtualKeyCode::Right),
            })
        );
        assert_eq!(
            bindings
                .insert_axis(
                    TEST_CONTROLLER_AXIS,
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::RightX,
                        invert: false,
                        dead_zone: 0.25,
                    },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings.remove_axis(&TEST_CONTROLLER_AXIS),
            Some(Axis::Controller {
                controller_id: 0,
                axis: ControllerAxis::RightX,
                invert: false,
                dead_zone: 0.25,
            })
        );
        assert_eq!(
            bindings
                .insert_axis(TEST_MOUSEWHEEL_AXIS, Axis::MouseWheel { horizontal: false },)
                .unwrap(),
            None
        );
        assert_eq!(
            bindings.remove_axis(&TEST_MOUSEWHEEL_AXIS),
            Some(Axis::MouseWheel { horizontal: false })
        );
    }
}
