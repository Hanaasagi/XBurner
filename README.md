<img src="assets/xburner.png">

# A multi-mode, rule-based keyboard customizer

## Features
- Key Remapping
- Execute command

## Install

```
$ git clone https://github.com/Hanaasagi/XBurner
$ cd XBurner
$ cargo install --path .
```
## Usage

```
USAGE:
    XBurner [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
        --silent     Suppress output of all key events
    -v, --verbose
    -V, --version    Print version information

SUBCOMMANDS:
    echo           Echo key infomation that you typed
    help           Print this message or the help of the given subcommand(s)
    list-device    List devices infomation of this computer
    list-keys      List supported keys reported by the device
    run
```

## Configuration

See `exmaple.yml`.

## Q&A

#### How to know device path?

```
$ XBurner list-device
Trying to scan all of /dev/input
Available devices:
/dev/input/event0   : Power Button
/dev/input/event1   : Lid Switch
/dev/input/event2   : AT Translated Set 2 keyboard
/dev/input/event3   : Video Bus
/dev/input/event4   : PC Speaker
/dev/input/event5   : Ideapad extra buttons
/dev/input/event6   : CUST0001:00 06CB:CE44 Mouse
/dev/input/event7   : CUST0001:00 06CB:CE44 Touchpad
/dev/input/event8   : HD-Audio Generic HDMI/DP,pcm=3
/dev/input/event9   : HD-Audio Generic HDMI/DP,pcm=7
/dev/input/event10  : HD-Audio Generic HDMI/DP,pcm=8
/dev/input/event11  : HD-Audio Generic Mic
/dev/input/event12  : HD-Audio Generic Headphone
/dev/input/event13  : PixArt USB Optical Mouse
/dev/input/event14  : Topre Corporation HHKB Professional
/dev/input/event15  : Integrated Camera: Integrated C
/dev/input/event16  : XBurner
```


#### How to know key name? 

Just run `XBurner echo --device <your device path>` and press keyboard.

```
$ XBurner echo --device <your device path>
Timestamp: 1640964235146         PRESS          Kind: Key(KEY_A)
aTimestamp: 1640964235226        RELEASE        Kind: Key(KEY_A)
```


#### Start via systemd

Systemd Unit File

```
[Unit]
Description=xburner

[Service]
Type=simple
KillMode=process
WorkingDirectory=<YOUR HOME>
ExecStart=/usr/bin/XBurner run --config <CONFIG_PATH> --device <YOUR DEVICE NAME>
Restart=on-failure
RestartSec=3

# Maybe needed
Environment=DISPLAY=:0
Environment=RUST_BACKTRACE=1


[Install]
WantedBy=default.target
```

Because a process inherits the cgroup information from its parent process.
when you use xburner to execute shell commands, it will be in the same cgroup as xburner.
This is not a problem if you have no resource limitations on the process.
If you need to limit resources, you can use `systemd-run` to execute the shell.
For example, `systemd-run --slice <YOUR SLICE> --unit <UNIT NAME> --scope --user <SHELL_COMMAND>`.


## License

GNU General Public License v3.0
