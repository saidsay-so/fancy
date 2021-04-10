% fancy(1) | Fancy CLI

NAME
====

fancy - Fancy CLI

SYNOPSIS
========

`fancy get [speeds | temps | config]`

`fancy set [-f FAN_SPEED [FAN_SPEEDS ...] | -a] [-c CONFIGURATION]`

`fancy list [--recommended]`

DESCRIPTION
===========

fancy is the CLI of *fancy(7)*,
a set of software which allows to control laptop fans.

OPTIONS
=======

#### SET

`-f, --fans-speeds FAN_SPEEDS...`

: Set fans speeds by percentage, between 0 and 100


`-c, --config CONFIGURATION`

: Set the configuration used by the daemon


`-a, --auto`
: Let the daemon automatically choose the speed, according to the temperature

#### GET

`fancy get speeds`

: Get fans speeds

`fancy get temps`

: Get temperatures

`fancy get config`

: Get current configuration

#### LIST

List all available configurations

`--recommended`

: List only recommended configurations

BUGS
====

Bugs can be reported at https://github.com/MusiKid/fancy
