# `ibsc` example

```bash
#! /bin/sh

# Startup scripts {{{
xrandr --output HDMI-1 --right-of HDMI-0
sxhkd &
numlockx on &
autocutsel -selection PRIMARY -fork &
compton &
xautolock -time 30 -locker 'lock' -corners 0000 &
copyq &

feh --randomize --bg-fill ~/Pictures/Wallpapers/*
# }}}

# Monitor configuration {{{
ibsc monitor HDMI-0 -n left -d I II III IV V
ibsc monitor HDMI-1 -n right -d VI VII IIX IX X
# }}}

# Global configuration {{{
ibsc config window_gap 18
ibsc config focus_follows_pointer true
ibsc config left_padding 0
ibsc config right_padding 0
ibsc config bottom_padding 0
ibsc config top_padding 10
ibsc config -m left top_padding 0
# }}}

# Internal rules {{{
ibsc rule -a git-cola --state floating
ibsc rule -a gnome-calculator --state floating
ibsc rule -a copyq --state floating
# }}}

# vim: syn=sh
```