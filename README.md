# abletime

[![CI](https://github.com/adamweiner/abletime/workflows/CI/badge.svg)](![CI](https://github.com/adamweiner/abletime/workflows/CI/badge.svg?branch=master))
[![MIT license](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/adamweiner/abletime/blob/master/LICENSE)

## What?

`abletime` is a utility meant for calculating time spent on projects by inspecting creation and modification timestamps
of project files in a particular directory.

## Why?

~~An excuse to learn Rust :)~~

This tool is really only useful for nerds (like [me](https://github.com/adamweiner)) that have let semantic versioning
influence workflows outside of programming projects (i.e. Ableton projects, where the project is represented by a
single file, not a repository full of files). Just like code under version control, there are lots of benefits to this
for other types of projects: ease of going back to an old version, history of changes made over time, and freedom to
delete things that you don't need in newer versions but may want to reference at some point in the future.

Suggested approach to semantic versioning for project files:
* Every new save is a patch version
* Every new session is a minor version
* Major versions denote project milestones or significant changes in direction

If a project is structured according to the following basic rules, `abletime` will be useful:

* Project files are all saved in the same directory
* Every new version is saved as a copy (ideally with a new semantic version in the name)

For example:
```
$ ls -l ~/Ableton\ Projects/abletime\ Project
-rw-r--r--@ 1 adam  staff   302K May 22 11:09 abletime 0.0.1.als
-rw-r--r--@ 1 adam  staff   407K May 22 11:40 abletime 0.0.2.als
-rw-r--r--@ 1 adam  staff   423K May 22 11:46 abletime 0.0.3.als
-rw-r--r--@ 1 adam  staff   538K May 22 12:04 abletime 0.0.4.als
-rw-r--r--@ 1 adam  staff   537K May 22 16:53 abletime 0.1.0.als
-rw-r--r--@ 1 adam  staff   576K May 22 17:18 abletime 0.1.1.als
```

The inspiration for `abletime` came from trying to quantify the black hole that can be time spent working on a song, but
assuming the structure above is used, it can apply to any kind of project. Just tell `abletime` what kind of files to
look for.

## Usage

```
USAGE:
    abletime [OPTIONS] [directory]

ARGS:
    <directory>    Directory to inspect. Defaults to current directory [default: .]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -m, --max-minutes-between-saves <max-minutes-between-saves>
            Maximum number of minutes allowed between saves for time to be counted. Values <= 0 will disable this
            feature [default: 60]
    -s, --suffix <suffix>
            Project file suffix. Default value works for Ableton projects [default: .als]
```

```
$ ./abletime ~/Ableton\ Projects/abletime\ Project
Start time            Duration      Name
Fri May 22 11:09:15   0:31:00.002   abletime 0.0.1.als
Fri May 22 11:40:15   0:06:38.036   abletime 0.0.2.als
Fri May 22 11:46:53   0:18:04.473   abletime 0.0.3.als
Fri May 22 12:04:58   0:00:00.443   abletime 0.0.4.als
Fri May 22 16:53:06   0:25:06.602   abletime 0.1.0.als
Fri May 22 17:18:12   0:00:00.475   abletime 0.1.1.als

Total project time
1:20:50.033
```

## Implementation details

* In order to avoid overestimating, a maximum amount of time between saves is used as a rough way to identify project
files that represent the end of a session. Durations for these versions fall back to `modified time - created
time`. In the example above, they are less than 1s.

## Things to implement, maybe

* Strict semantic versioning mode which requires a version in every project file name and relies on semver for
session boundaries, not elapsed time between saves
* Visualizations
