# rover-revolution
Rust client for Brookstone Rover Revolution


![image of rover box](https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fcontent.propertyroom.com%2Flistings%2Fsellers%2Fseller1%2Fimages%2Forigimgs%2Fbrookstone-rover-revolution-wireless-spy-vehicle-with-app-control-1_272020181562411604.jpg&f=1&nofb=1&ipt=dc0b7dac171bf3797c2ecb434f110c7277282abd5682398c792658c83ec0624f&ipo=images)

This program can control the Brookstone Rover Revolution via WiFi. It uses the reverse engineered protocol from the android app which is not available in the play store anymore. The camera feed is displayed using SDL2.

Keybindings:
- `q` to quit
- `wasd` to move
- `e` to toggle "stealth mode" aka. infrared lights
- `1` or `2` to toggle between driving and turret camera
- `arrow keys` to move the turret

> Since the raw socket protocol is reverse engineered, the program might panic when encoutering unexpected data. This is still a work in progress.

Manual: https://manuals.brookstone.com/851135p_manual.pdf