# mb2-embassy-rgb: blinky + RGB example for mb2
Bart Massey 2024

This crate demos Embassy on the MicroBit v2. 

* Blinks an LED on the MB2 with a one-second cycle time
* Blinks an external RGB LED, cycling through the colors at
  one per second
* Reads a knob every half second and prints the value via
  RTT

Connect the RGB LED to the MB2 as follows:

* Red to P9 (GPIO1)
* Green to P8 (GPIO2)
* Blue to P16 (GPIO3)
* Gnd to Gnd

Connect the potentiometer (knob) to the MB2 as follows:

* Pin 1 to +3.3V
* Pin 2 to P2
* Pin 3 to Gnd
