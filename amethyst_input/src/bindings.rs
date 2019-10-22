//! Defines binding structure used for saving and loading input settings.

use std::{
    borrow::Borrow,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
};

use derivative::Derivative;
use fnv::FnvHashMap as HashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::{Axis, Button};

/// Define a set of types used for bindings configuration.
/// Usually defaulted to `StringBindings`, which uses `String`s.
///
/// By defining your own set of types (usually enums),
/// you will be able to have compile-time guarantees while handling events,
/// and you can also add additional context, for example player index
/// in local multiplayer game.
///
/// Example configuration for local multiplayer driving game might look like this:
/// ```rust,edition2018,no_run,noplaypen
/// # use serde::{Serialize, Deserialize};
/// # use amethyst_input::{BindingTypes, Bindings};
/// type PlayerId = u8;
///
/// #[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
/// enum AxisBinding {
///     Throttle(PlayerId),
///     Steering(PlayerId),
/// }
///
/// #[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
/// enum ActionBinding {
///     UsePowerup(PlayerId),
/// }
///
/// #[derive(Debug)]
/// struct DriverBindingTypes;
/// impl BindingTypes for DriverBindingTypes {
///     type Axis = AxisBinding;
///     type Action = ActionBinding;
/// }
///
/// type GameBindings = Bindings<DriverBindingTypes>;
/// ```
/// And the `bindings.ron`:
/// ```ron
/// (
///   axes: {
///     Throttle(0): Emulated(pos: Key(W), neg: Key(S)),
///     Steering(0): Emulated(pos: Key(D), neg: Key(A)),
///     Throttle(1): Emulated(pos: Key(Up), neg: Key(Down)),
///     Steering(1): Emulated(pos: Key(Right), neg: Key(Left)),
///   },
///   actions: {
///     UsePowerup(0): [[Key(E)]],
///     UsePowerup(1): [[Key(P)]],
///   },
/// )
/// ```
pub trait BindingTypes: Debug + Send + Sync + 'static {
    /// Type used for defining axis keys. Usually an enum or string.
    type Axis: Clone + Debug + Hash + Eq + Send + Sync + 'static;
    /// Type used for defining action keys. Usually an enum or string.
    type Action: Clone + Debug + Hash + Eq + Send + Sync + 'static;
}

/// The builtin `BindingTypes` implementation, set of types for binding configuration keys.
/// Uses `String` for both axes and actions. Usage of this type is discouraged
/// and it's meant mainly for prototypes. Check `BindingTypes` for examples.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct StringBindings;

impl BindingTypes for StringBindings {
    type Axis = String;
    type Action = String;
}

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
///         "leftright": Emulated(
///             pos: Key(Right),
///             neg: Key(Left)
///         )
///     },
///     actions: {
///         "fire": [ [Mouse(Left)], [Key(X)] ], // Multiple bindings for one action
///         "reload": [ [Key(LControl), Key(R)] ] // Combinations of multiple bindings possible
///     }
/// )
/// ```
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug(bound = ""), Default(bound = ""), Clone(bound = ""))]
#[serde(bound(
    serialize = "T::Axis: Serialize, T::Action: Serialize",
    deserialize = "T::Axis: Deserialize<'de>, T::Action: Deserialize<'de>",
))]
pub struct Bindings<T: BindingTypes> {
    pub(super) axes: HashMap<T::Axis, Axis>,
    /// The inner array here is for button combinations, the other is for different possibilities.
    ///
    /// So for example if you want to quit by either "Esc" or "Ctrl+q" you would have
    /// `[[Esc], [Ctrl, Q]]`.
    pub(super) actions: HashMap<T::Action, SmallVec<[SmallVec<[Button; 2]>; 4]>>,
}

/// An enum of possible errors that can occur when binding an action or axis.
#[derive(Clone, Derivative)]
#[derivative(Debug(bound = ""))]
pub enum BindingError<T: BindingTypes> {
    /// Axis buttons have overlap with an action combo of length one.
    AxisButtonAlreadyBoundToAction(T::Action, Button),
    /// Axis buttons provided have overlap with an existing axis.
    AxisButtonAlreadyBoundToAxis(T::Axis, Axis),
    /// A combo of length one was provided, and it overlaps with an axis binding.
    ButtonBoundToAxis(T::Axis, Axis),
    /// Combo provided was already bound to the contained action.
    ComboAlreadyBound(T::Action),
    /// Combo provided for action binding has two (or more) of the same button.
    ComboContainsDuplicates(T::Action),
    /// That specific axis on that specific controller is already in use for an
    /// axis binding.
    ControllerAxisAlreadyBound(T::Axis),
    /// The given axis was already bound for use
    MouseAxisAlreadyBound(T::Axis),
    /// You attempted to bind a mousewheel axis twice.
    MouseWheelAxisAlreadyBound(T::Axis),
}

impl<T: BindingTypes> PartialEq for BindingError<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BindingError::ComboContainsDuplicates(a),
                BindingError::ComboContainsDuplicates(x),
            ) => a == x,
            (BindingError::ComboAlreadyBound(a), BindingError::ComboAlreadyBound(x)) => a == x,
            (BindingError::ButtonBoundToAxis(a, b), BindingError::ButtonBoundToAxis(x, y)) => {
                a == x && b == y
            }
            (
                BindingError::AxisButtonAlreadyBoundToAxis(a, b),
                BindingError::AxisButtonAlreadyBoundToAxis(x, y),
            ) => a == x && b == y,
            (
                BindingError::AxisButtonAlreadyBoundToAction(a, b),
                BindingError::AxisButtonAlreadyBoundToAction(x, y),
            ) => a == x && b == y,
            (
                BindingError::ControllerAxisAlreadyBound(a),
                BindingError::ControllerAxisAlreadyBound(x),
            ) => a == x,
            (
                BindingError::MouseWheelAxisAlreadyBound(a),
                BindingError::MouseWheelAxisAlreadyBound(x),
            ) => a == x,
            (_, _) => false,
        }
    }
}

impl<T: BindingTypes> Display for BindingError<T>
where
    T::Action: Display,
    T::Axis: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            BindingError::ComboContainsDuplicates(ref id) => write!(
                f,
                "Combo provided contained two (or more) of the same button: {}",
                id
            ),
            BindingError::ComboAlreadyBound(ref action) => {
                write!(f, "Combo provided was already bound to action {}", action)
            }
            BindingError::ButtonBoundToAxis(ref id, ref _axis) => {
                write!(f, "Button provided was a button in use by axis {}", id)
            }
            BindingError::AxisButtonAlreadyBoundToAxis(ref id, ref _axis) => write!(
                f,
                "Axis provided contained a button that's already in use by axis {}",
                id
            ),
            BindingError::AxisButtonAlreadyBoundToAction(ref id, ref _action) => write!(
                f,
                "Axis provided contained a button that's already in use by single button action {}",
                id
            ),
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

impl<T: BindingTypes> Error for BindingError<T>
where
    T::Action: Display,
    T::Axis: Display,
{
}

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

impl<T: BindingTypes> Bindings<T> {
    /// Creates a new empty Bindings structure
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: BindingTypes> Bindings<T> {
    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<A: Into<T::Axis>>(
        &mut self,
        id: A,
        axis: Axis,
    ) -> Result<Option<Axis>, BindingError<T>> {
        let id = id.into();
        self.check_axis_invariants(&id, &axis)?;
        Ok(self.axes.insert(id, axis))
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<A>(&mut self, id: &A) -> Option<Axis>
    where
        T::Axis: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.axes.remove(id)
    }

    /// Returns a reference to an axis.
    pub fn axis<A>(&self, id: &A) -> Option<&Axis>
    where
        T::Axis: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.axes.get(id)
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> impl Iterator<Item = &T::Axis> {
        self.axes.keys()
    }

    /// Add a button or button combination to an action.
    ///
    /// This will attempt to insert a new binding between this action and the button(s).
    pub fn insert_action_binding<B: IntoIterator<Item = Button>>(
        &mut self,
        id: T::Action,
        binding: B,
    ) -> Result<(), BindingError<T>> {
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
            bindings.push(bind.clone());
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
        T::Action: Borrow<A>,
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
        T::Action: Borrow<A>,
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
    pub fn actions(&self) -> impl Iterator<Item = &T::Action> {
        self.actions.keys()
    }

    /// Check that this structure upholds its guarantees. Should only be necessary when serializing or deserializing the bindings.
    pub fn check_invariants(&mut self) -> Result<(), BindingError<T>> {
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

    fn check_action_invariants(
        &self,
        id: &T::Action,
        bind: &[Button],
    ) -> Result<(), BindingError<T>> {
        // Guarantee each button is unique.
        for i in 0..bind.len() {
            for j in (i + 1)..bind.len() {
                if bind[i] == bind[j] {
                    return Err(BindingError::ComboContainsDuplicates(id.clone()));
                }
            }
        }
        if bind.len() == 1 {
            for (k, a) in self.axes.iter() {
                if let Axis::Emulated { pos, neg } = a {
                    if bind[0] == *pos || bind[0] == *neg {
                        return Err(BindingError::ButtonBoundToAxis(k.clone(), a.clone()));
                    }
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

    fn check_axis_invariants(&self, id: &T::Axis, axis: &Axis) -> Result<(), BindingError<T>> {
        match axis {
            Axis::Emulated {
                pos: ref axis_pos,
                neg: ref axis_neg,
            } => {
                for (k, a) in self.axes.iter().filter(|(k, _a)| *k != id) {
                    if let Axis::Emulated { pos, neg } = a {
                        if axis_pos == pos || axis_pos == neg || axis_neg == pos || axis_neg == neg
                        {
                            return Err(BindingError::AxisButtonAlreadyBoundToAxis(
                                k.clone(),
                                a.clone(),
                            ));
                        }
                    }
                }
                for (k, a) in self.actions.iter() {
                    for c in a {
                        // Since you can't bind combos to an axis we only need to check combos with length 1.
                        if c.len() == 1 && (c[0] == *axis_pos || c[0] == *axis_neg) {
                            return Err(BindingError::AxisButtonAlreadyBoundToAction(
                                k.clone(),
                                c[0],
                            ));
                        }
                    }
                }
            }
            Axis::Controller {
                controller_id: ref input_controller_id,
                axis: ref input_axis,
                ..
            } => {
                for (k, a) in self.axes.iter().filter(|(k, _a)| *k != id) {
                    if let Axis::Controller {
                        controller_id,
                        axis,
                        ..
                    } = a
                    {
                        if controller_id == input_controller_id && axis == input_axis {
                            return Err(BindingError::ControllerAxisAlreadyBound(k.clone()));
                        }
                    }
                }
            }
            Axis::Mouse { axis, .. } => {
                for (k, a) in self.axes.iter().filter(|(k, _a)| *k != id) {
                    if let Axis::Mouse {
                        axis: mouse_axis, ..
                    } = a
                    {
                        if axis == mouse_axis {
                            return Err(BindingError::MouseAxisAlreadyBound(k.clone()));
                        }
                    }
                }
            }
            Axis::MouseWheel {
                horizontal: ref input_horizontal,
            } => {
                for (k, a) in self.axes.iter().filter(|(k, _a)| *k != id) {
                    if let Axis::MouseWheel { horizontal } = a {
                        if input_horizontal == horizontal {
                            return Err(BindingError::MouseWheelAxisAlreadyBound(k.clone()));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{button::*, controller::ControllerAxis};
    use winit::{MouseButton, VirtualKeyCode};

    #[test]
    fn add_and_remove_actions() {
        let mut bindings = Bindings::<StringBindings>::new();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings("test_action").next(), None);

        bindings
            .insert_action_binding(
                String::from("test_action"),
                [Button::Mouse(MouseButton::Left)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(
            bindings.actions().collect::<Vec<_>>(),
            vec![&String::from("test_action")]
        );
        let action_bindings = bindings.action_bindings("test_action").collect::<Vec<_>>();
        assert_eq!(action_bindings, vec![[Button::Mouse(MouseButton::Left)]]);
        bindings
            .remove_action_binding("test_action", &[Button::Mouse(MouseButton::Left)])
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings("test_action").next(), None);

        bindings
            .insert_action_binding(
                String::from("test_action"),
                [
                    Button::Mouse(MouseButton::Left),
                    Button::Mouse(MouseButton::Right),
                ]
                .iter()
                .cloned(),
            )
            .unwrap();
        assert_eq!(
            bindings.actions().collect::<Vec<_>>(),
            vec![&String::from("test_action")],
        );
        let action_bindings = bindings.action_bindings("test_action").collect::<Vec<_>>();
        assert_eq!(
            action_bindings,
            vec![[
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right)
            ]]
        );
        bindings
            .remove_action_binding(
                "test_action",
                &[
                    Button::Mouse(MouseButton::Left),
                    Button::Mouse(MouseButton::Right),
                ],
            )
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings("test_action").next(), None);

        bindings
            .insert_action_binding(
                String::from("test_action"),
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
                    "test_action",
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
                .remove_action_binding("test_action", &[Button::Mouse(MouseButton::Left),],)
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
        assert_eq!(actions, vec![&String::from("test_action")]);
        let action_bindings = bindings.action_bindings("test_action").collect::<Vec<_>>();
        assert_eq!(
            action_bindings,
            vec![[
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right)
            ]]
        );
        bindings
            .remove_action_binding(
                "test_action",
                &[
                    Button::Mouse(MouseButton::Right),
                    Button::Mouse(MouseButton::Left),
                ],
            )
            .unwrap();
        assert_eq!(bindings.actions().next(), None);
        assert_eq!(bindings.action_bindings("test_action").next(), None);
    }
    #[test]
    fn insert_errors() {
        let mut bindings = Bindings::<StringBindings>::new();
        assert_eq!(
            bindings
                .insert_action_binding(
                    String::from("test_action"),
                    [
                        Button::Mouse(MouseButton::Left),
                        Button::Mouse(MouseButton::Right),
                        Button::Mouse(MouseButton::Left),
                    ]
                    .iter()
                    .cloned(),
                )
                .unwrap_err(),
            BindingError::ComboContainsDuplicates(String::from("test_action"))
        );
        bindings
            .insert_action_binding(
                String::from("test_action"),
                [Button::Mouse(MouseButton::Left)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(
            bindings
                .insert_action_binding(
                    String::from("test_action"),
                    [Button::Mouse(MouseButton::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ComboAlreadyBound(String::from("test_action"))
        );
        assert_eq!(
            bindings
                .insert_action_binding(
                    String::from("test_action_2"),
                    [Button::Mouse(MouseButton::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ComboAlreadyBound(String::from("test_action"))
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_axis"),
                    Axis::Emulated {
                        pos: Button::Mouse(MouseButton::Left),
                        neg: Button::Mouse(MouseButton::Right),
                    },
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAction(
                String::from("test_action"),
                Button::Mouse(MouseButton::Left)
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_axis"),
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
                    String::from("test_action_2"),
                    [Button::Key(VirtualKeyCode::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ButtonBoundToAxis(
                String::from("test_axis"),
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_action_binding(
                    String::from("test_axis_2"),
                    [Button::Key(VirtualKeyCode::Left),].iter().cloned(),
                )
                .unwrap_err(),
            BindingError::ButtonBoundToAxis(
                String::from("test_axis"),
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_axis_2"),
                    Axis::Emulated {
                        pos: Button::Key(VirtualKeyCode::Left),
                        neg: Button::Key(VirtualKeyCode::Up),
                    },
                )
                .unwrap_err(),
            BindingError::AxisButtonAlreadyBoundToAxis(
                String::from("test_axis"),
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Left),
                    neg: Button::Key(VirtualKeyCode::Right),
                }
            )
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_controller_axis"),
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
                    String::from("test_controller_axis"),
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
                    String::from("test_controller_axis_2"),
                    Axis::Controller {
                        controller_id: 0,
                        axis: ControllerAxis::LeftX,
                        invert: true,
                        dead_zone: 0.1,
                    },
                )
                .unwrap_err(),
            BindingError::ControllerAxisAlreadyBound(String::from("test_controller_axis"))
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_mouse_wheel_axis"),
                    Axis::MouseWheel { horizontal: true },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_mouse_wheel_axis"),
                    Axis::MouseWheel { horizontal: false },
                )
                .unwrap(),
            Some(Axis::MouseWheel { horizontal: true })
        );
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_mouse_wheel_axis_2"),
                    Axis::MouseWheel { horizontal: false },
                )
                .unwrap_err(),
            BindingError::MouseWheelAxisAlreadyBound(String::from("test_mouse_wheel_axis"))
        );
    }

    #[test]
    fn add_and_remove_axes() {
        let mut bindings = Bindings::<StringBindings>::new();
        assert_eq!(
            bindings
                .insert_axis(
                    String::from("test_axis"),
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
                    String::from("test_controller_axis"),
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
            bindings.remove_axis("test_controller_axis"),
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
                    String::from("test_mouse_wheel_axis"),
                    Axis::MouseWheel { horizontal: false },
                )
                .unwrap(),
            None
        );
        assert_eq!(
            bindings.remove_axis("test_mouse_wheel_axis"),
            Some(Axis::MouseWheel { horizontal: false })
        );
    }
}
