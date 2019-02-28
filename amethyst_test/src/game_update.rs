use amethyst::prelude::*;

/// Exposes the `update` method of game data so that this crate's `State`s can invoke it.
pub trait GameUpdate {
    /// Runs the systems to update the game.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` in which the game takes place.
    fn update(&mut self, _world: &World) {}
}

// Implement for built-in Amethyst `GameData`
impl<'a, 'b> GameUpdate for GameData<'a, 'b> {
    fn update(&mut self, world: &World) {
        GameData::update(self, world);
    }
}
