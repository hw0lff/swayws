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
  swap   Swaps two workspaces with each other
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

### `swayws s[wap]`
```
Swaps two workspaces with each other

Usage: swayws swap <WORKSPACE> <WORKSPACE>
```

### `swayws f[ocus]`
```
Focus a workspace

Usage: swayws focus [OPTIONS] <WORKSPACE>

Arguments:
  <WORKSPACE>  Workspace to focus

Options:
      --name   Focus the workspace by name. This is the default action if no flag is specified
      --num    Focus the workspace by num
      --id     Focus the workspace by id
      --smart  Try to focus the workspace a bit smartly
```

## Examples
```sh
swayws move 1 eDP-1

swayws range 11 20 DP-3
swayws range 11 20 eDP-1 --away
swayws range 11 20 eDP-1 --away --not DP-3 --not DP-5

swayws list

swayws focus 1
swayws focus --smart 4

swayws swap 4 17
```

