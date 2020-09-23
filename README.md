# Improved Binary Space Window Manager

![main workflow](https://github.com/cmizzi/ibswm/workflows/Continuous%20integration/badge.svg)

**WARNING: This application is under development and not stable yet.**

`ibswm` is a tiling window manager that represents windows as
the leaves of full binary tree. It only manages events sent from
X server and messages it received from the socket communication.
`ibsc` is a program that writes messages into the `ibswm`'s
socket. This window manager doesn't handle any keyboard or mouse
events. You need a third party program (like `sxhkd`) to communicate
informations between your action and the window manager, through
`ibsc` commands.

This project is strongly inspired by [`bspwm`](https://github.com/baskerville/bspwm)
and tries to improve it. It is also entirely written in [Rust](https://www.rust-lang.org/)
to enable memory-safety and be able to work with higher performance.

## Installation

*TODO*

## Configuration

You can easily configure `ibswm` behavior using bash file stored into
`$HOME/.config/ibswm/ibswmrc` file. This file is executed directly right
after the window manager is booted. It can handle any bash command to start
when the window manager is executed. Also, this file is a perfect place
to configure the window manager through `ibsc` command like :

```bash
ibsc config window_gap 18
ibsc config focus_follows_pointer true
```

### `ibsc monitor`

Configure a monitor. To configure a monitor, you must known the
monitor ID, given by `xrandr --query`. You can have an unlimited number
of desktops by monitor. By default, if you did not configure a monitor,
the window manager will try to detect informations for you. Also, you can
give a name to a monitor in order to apply some configuration just for
this monitor.

```bash
ibsc monitor MONITOR_ID --name NAME --desktops DESKTOP_0 DESKTOP_1 DESKTOP_N
```

### `ibsc config`

Configure a monitor. Every command listed here can use the `--monitor`
argument to only configure a monitor and not every of them.

| Key | Value | Description |
| --- | --- | --- |
| `window_gap` | `int` | Gap between window on the same desktop. |
| `focus_follows_pointer` | `bool` | Focus when hovering window. |

### `ibsc rule`

Configure a rule for a particular application. This command is useful
when you want an application to always map a specific monitor, or to be
started as float by default.

```bash
bspc rule --application git-cola --state STATE --desktop DESKTOP_0
```

To know which name an application have, you can start your application
and execute this command :

```
xwininfo -root -children
```

## How it works

`ibswm` works using a [binary tree](https://en.wikipedia.org/wiki/Binary_tree#:~:text=In%20computer%20science%2C%20a%20binary,child%20and%20the%20right%20child.).

*TODO: Implement this part of documentation.*

## Supported protocols and standards

`ibswm` tries to follow as much as possible the following protocols :

- [RandR](https://www.x.org/wiki/Projects/XRandR/) and [Xinerama](https://en.wikipedia.org/wiki/Xinerama) ;
- [EWMH](https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints) and [ICCCM](https://en.wikipedia.org/wiki/Inter-Client_Communication_Conventions_Manual) standards ;

## Development

To improve this application, you need to install some
dependencies into order to test your modification. You'll
need the following packages installed into your distribution :

- `xserver-xephyr`

```bash
sudo apt install xserver-xephyr
```

Now, you can clone the repository and start working on it.

```bash
# Start a background process to see your modification.
Xephyr -screen 640x480 +extension RANDR -ac +xinerama :1 &

# Start the window manager and map the display to Xephyr server.
DISPLAY=:1 cargo run -- -vv

# Start an application and map it to Xephyr display.
DISPLAY=:1 alacritty
```