# Contributing
Contributions and useful criticism are welcomed! There is maybe a feature that you have implemented or thinking of, bugs that haven't been spotted, code which can be improved, etc.

## Projects
Fancy is subdivised between 3 parts: 
- the service daemon, which treats requests from D-Bus, write them to the EC and controls the EC according to the configuration.
- the CLI, which provides control/view over the D-Bus interface.
- the GUI, same as the CLI but graphical.

You can directly fork the project you want to work on or fork this repository and all the others and change the submodules remotes.

## Issues

If you encounter a bug, or have an idea to improve Fancy, you can open an issue. Don't forget to check if there isn't a similar issue already opened/closed.

## Pull request

#### PLEASE DON'T SUBMIT PULL REQUESTS TO THIS PROJECT! You have to open them on the project you work on (CLI, GUI or service).

It is recommended to open an issue before starting to write a pull request (except if your pull request close an existing issue, of course). We can then discuss of the most correct implementation, backward compatibility, etc.


### Code formatting
Before sumbitting a pull request, you have to ensure that commits are correctly formatted using `cargo fmt --all`.
