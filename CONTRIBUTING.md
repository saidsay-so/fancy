# Contributing

Contributions and useful criticism are welcomed! There is maybe a feature that you have implemented or thinking of, bugs that haven't been spotted, code which can be improved, etc.

## Projects

Fancy is subdivised between 3 parts:
- the service daemon, which treats requests from D-Bus, write them to the EC and controls the EC according to the configuration.
- the CLI, which provides control/view over the D-Bus interface.
- the GUI, same as the CLI but graphical.

## Issues

If you encounter a bug, or have an idea to improve Fancy, you can open an issue. Don't forget to check if there isn't a similar issue already opened/closed.

## Pull request

### Code formatting

Before sumbitting a pull request, you have to ensure that commits are correctly formatted using `cargo fmt --all`.
