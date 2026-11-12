# Grafix

Grafix is meant to be an easy to use library for SPI-based Displays.
It draws directly to the Linux frame buffer, without the need for any desktop environment or window manager.

It has been tested on a single no-name 3.5 inch RPi compatible SPI display powered by a Raspberry Pi Zero W running Raspbian Bookworm Lite, so it should hopefully work smoothly on all newer models.

Disclaimer: This setup is meant for a Raspberry Pi Lite system with no xorg or wayland installed, or any other application that would attempt to use the frame buffer. It has NOT been tested on any other OSes. Use only if you have debugging time to spare.

This repo is part of a larger project I'm working on called Mosaic.


## Installation

The only prerequisite is the existence of a frame buffer device, which you can check with 
```sh
cat /dev/fb*
```
 
If that shows nothing,
open the `/boot/firmware/config.txt` file, and under the `[all]` section, paste the following
```sh
dtparam=spi=on
dtoverlay=piscreen2r,rotate=270
```
amd then rebooot. 

Type 

```sh
ls /dev/fb*
```
to see all available frame buffers.

To test out the display, run

```sh
cat /dev/urandom > /dev/fb0 # could be fb1 or other number
```

You should see colourful noise appear (rather slowly) on the display.


## Example

TODO
