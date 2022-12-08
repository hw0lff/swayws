# SwayWs
A tool which allows easy moving of workspaces to and from outputs

Developed for use with the [Sway](https://swaywm.org/) compositor

## Features
- Move numeric ranges of workspaces at once
- When a moved workspace is focused, it will always be opened at the specified output
- The focus is returned by default to the workspace that was focused before SwayWs was invoked
- The previously visible workspaces are visible again by default after moving one or more workspaces

## Usage

```
SwayWs allows easy moving of workspaces to and from outputs

Usage: swayws <COMMAND>

Commands:
  focus  Focus a workspace
  list   Lists infos about sway
  move   Moves a workspace to a specified output
  range  Moves a range of workspaces to a specified output
```

### `swayws m[ove]`
```
Moves a workspace to a specified output

Usage: swayws move [OPTIONS] <WORKSPACE> <OUTPUT>

Arguments:
  <WORKSPACE>  Workspace to move
  <OUTPUT>     Name of the output

Options:
  -a, --away       Moves workspace to output that does not match the specified output name
      --not <NOT>  Excludes outputs to move workspace to, must be used with --away
  -f, --focus      Focuses specified workspace after moving it
```

### `swayws r[ange]`
```
Moves a range of workspaces to a specified output

Usage: swayws range [OPTIONS] <START> <END> <OUTPUT>

Arguments:
  <START>   First workspace in range
  <END>     Last workspace in range
  <OUTPUT>  Name of the output

Options:
  -a, --away       Moves workspace to output that does not match the specified output name
      --not <NOT>  Excludes outputs to move workspace to, must be used with --away
  -n, --numeric    Assumes <start> and <end> are numbers and binds all workspaces in between them to the specified output
```

## Examples
```sh
swayws move 1 eDP-1

swayws range 11 20 DP-3
swayws range 11 20 eDP-1 --away
swayws range 11 20 eDP-1 --away --not DP-3 --not DP-5

swayws list

swayws focus 1
```

