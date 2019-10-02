//! Systems and resources to have a consistent, separate simulation frame rate from the ECS
//! frame rate.

use amethyst_core::{
    ecs::{Read, System, Write},
    timing::Time,
};
use std::{ops::RangeInclusive, time::Duration};

/// Default number of network simulation frames per second.
const DEFAULT_SIM_FRAME_RATE: u32 = 30;

/// This system is used exclusively to update the state of the `NetworkSimulationTime` resource.
pub struct NetworkSimulationTimeSystem;

impl<'s> System<'s> for NetworkSimulationTimeSystem {
    type SystemData = (Write<'s, NetworkSimulationTime>, Read<'s, Time>);

    fn run(&mut self, (mut sim_time, game_time): Self::SystemData) {
        sim_time.update_elapsed(game_time.delta_time());
        sim_time.reset_frame_lag();
        while sim_time.elapsed_duration() > sim_time.per_frame_duration() {
            sim_time.increment_frame_number();
        }
    }
}

/// Resource to track the state of the network simulation separately from the ECS frame timings
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkSimulationTime {
    /// The current simulation frame
    frame_number: u32,
    /// Accumulated duration since last simulation frame
    elapsed_duration: Duration,
    /// Duration per frame
    per_frame_duration: Duration,
    /// Determines how often we send messages. i.e. "Every N frames" where N is message_send_rate
    message_send_rate: u8,
    /// Number of frames behind the simulation is. This will usually be 0 or 1 if the ECS system
    /// is keeping up
    frame_lag: u32,
}

impl NetworkSimulationTime {
    /// Returns the simulation frame numbers needed to be run this game frame.
    pub fn sim_frames_to_run(&self) -> RangeInclusive<u32> {
        (self.frame_number + 1 - self.frame_lag)..=self.frame_number
    }

    /// Determines whether or not to send a message in the current frame based on the
    /// `message_send_rate`
    pub fn should_send_message_now(&self) -> bool {
        self.should_send_message(self.frame_number)
    }

    /// Determines whether or not to send a message based on the `message_send_rate`
    pub fn should_send_message(&self, frame: u32) -> bool {
        frame % u32::from(self.message_send_rate) == 0
    }

    /// Bumps the frame number
    pub fn increment_frame_number(&mut self) {
        self.frame_number += 1;
        self.elapsed_duration -= self.per_frame_duration;
        self.frame_lag += 1;
    }

    /// Resets the frame lag
    pub fn reset_frame_lag(&mut self) {
        self.frame_lag = 0;
    }

    /// Increase the `elapsed_duration` by the given duration
    pub fn update_elapsed(&mut self, duration: Duration) {
        self.elapsed_duration += duration;
    }

    /// Returns the current simulation frame number
    pub fn frame_number(&self) -> u32 {
        self.frame_number
    }

    /// Sets the frame number to the given frame number. This is useful when synchronizing frames
    /// with a server for example.
    pub fn set_frame_number(&mut self, new_frame: u32) {
        self.frame_number = new_frame;
    }

    /// Returns the total duration since the last simulation frame
    pub fn elapsed_duration(&self) -> Duration {
        self.elapsed_duration
    }

    /// The duration between each simulation frame. This number is calculated when a frame rate
    /// is set
    pub fn per_frame_duration(&self) -> Duration {
        self.per_frame_duration
    }

    /// The rate at which messages should be sent over the network.
    /// i.e. 'Every N frames' where N is `message_send_rate`.
    pub fn message_send_rate(&self) -> u8 {
        self.message_send_rate
    }

    /// The number of frames behind the simulation is. This will usually be 0 or 1 if the ECS system
    /// is keeping up
    pub fn frame_lag(&self) -> u32 {
        self.frame_lag
    }

    /// Set the rate at which the network simulation progresses. Specified in hertz (frames/second).
    pub fn with_sim_frame_rate(mut self, new_rate: u32) -> Self {
        self.per_frame_duration = Duration::from_secs(1) / new_rate;
        self
    }

    /// Set the rate which messages are sent. Specified as "every N frames" where N is new_rate.
    pub fn with_message_send_rate(mut self, new_rate: u8) -> Self {
        self.message_send_rate = new_rate;
        self
    }
}

impl Default for NetworkSimulationTime {
    fn default() -> Self {
        Self {
            frame_number: 0,
            elapsed_duration: Duration::from_secs(0),
            // Default to 30 frames / second
            per_frame_duration: Duration::from_secs(1) / DEFAULT_SIM_FRAME_RATE,
            // Default to sending a message with every simulation frame
            message_send_rate: 1,
            // Default the lag to run so systems have a chance to run on the frame 0
            frame_lag: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_calculated_properties_and_getters() {
        let time = NetworkSimulationTime::default().with_sim_frame_rate(20);
        assert_eq!(time.frame_number(), 0);
        assert_eq!(time.frame_lag(), 1);
        assert_eq!(time.message_send_rate(), 1);
        assert_eq!(time.per_frame_duration(), Duration::from_millis(50));
        assert_eq!(time.elapsed_duration(), Duration::from_millis(0));
    }

    #[test]
    fn test_message_send_rate_should_send_every_2_frames() {
        let time = NetworkSimulationTime::default().with_message_send_rate(2);

        for i in 1..100 {
            // every second frame (even) should return true
            if i % 2 == 0 {
                assert_eq!(time.should_send_message(i), true);
            } else {
                assert_eq!(time.should_send_message(i), false);
            }
        }
    }

    #[test]
    fn test_elapsed_duration_gets_updated() {
        let mut time = NetworkSimulationTime::default();

        let elapsed_time = Duration::from_millis(500);
        time.update_elapsed(elapsed_time);

        assert_eq!(time.elapsed_duration(), elapsed_time)
    }
}
