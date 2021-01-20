# Controlling System Execution

When writing a game you'll eventually reach a point where you want to have more control over when certain `System`s are executed, such as running them for specific `State`s or pausing them when a certain condition is met. Right now you have these three options to achieve said control:

- **Custom GameData:**

  Store multiple `Dispatcher`s in a custom `GameData`. Each `Dispatcher` has its own assigned `System`s and `State`s determines which `Dispatcher`s to run.

- **State-specific Dispatcher:**

  A `State` contains its own `Dispatcher` with its own `System`s and the `State` handles the execution.

- **Pausable Systems:**

  When registering a `System` with a `Dispatcher`, specify the value of a `Resource` `R`. The `System` runs only if the `Resource` equals that value. This allows for more selective enabling and disabling of `System`s.

This section contains guides that demonstrate each of these methods.
