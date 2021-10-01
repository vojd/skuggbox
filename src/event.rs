use winit::event::WindowEvent;

/// Generic trait for handling window events.
/// Mostly used to get the event types correct.
pub trait WindowEventHandler {
    /// Handles window events. The implementation should return true if the event was used
    /// otherwise false.
    /// * `self` - A mutable reference to the implementation
    /// * `event` - The event to react to
    fn handle_window_events(&mut self, event: &WindowEvent) -> bool;
}