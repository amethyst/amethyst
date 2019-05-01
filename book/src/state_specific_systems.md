# State-Specific Systems

When writing a game, there are systems that only apply to certain `State`s. These systems should only run when a particular state is active, and be ignored otherwise. There are multiple ways to do this:

* **Custom Game Data:**

    Store multiple dispatchers in a custom `GameData`, and each `State` determines which dispatchers to run.

* **State-specific dispatcher:**

    A state contains its own dispatcher with its own systems, and the state runs both the application dispatcher and its own dispatcher.

* **Pausable systems:**

    When registering a system with the application dispatcher, specify the value of a resource `R` that a system should run. The system runs only if the resource equals that value.

This section contains guides that demonstrate each of these methods.
