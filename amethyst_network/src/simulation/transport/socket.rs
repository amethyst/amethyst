use crate::simulation::requirements::DeliveryRequirement;

/// Trait defining the interface for the generic type that `SimulationTransportResource` requires.
pub trait Socket {
    fn default_requirement() -> DeliveryRequirement;
}
