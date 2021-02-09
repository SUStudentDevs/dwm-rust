# dwm-rust #
The goal of this project is to implement the [Dynamic Window Manager](https://dwm.suckless.org/) from suckless, using the Rust programming language.

We will try to use as many Rust "safe" features as possible, even if relying on the xlib library forces us to use unsafe C features in our code.

## Dependencies ##
* xlib and xft libraries (should be installed on any linux system)
* cargo and cargo-make to compile and install the project. Cargo comes installed with your typical Rust installation. Use `cargo install cargo-make`
* Xnest to test the wm in a nested Xserver environment

### Debian-based ###

```
sudo apt-get install --yes libfreetype6 libxft-dev
```

## Usage ##
* `cargo build` to build the project (`cargo build --release` to build it in a more optimized release mode)
* `cargo make xnest` to build the project and test it in a nested X environment
* `cargo make install` to build the project in release mode and install it in `/usr/local/bin`. This command will use sudo to copy the necessary files in `/usr/local/bin`, and therefore ask for your password.

## LICENSE ##
This project is under the GNU GPLv3 license. See more in [LICENSE](LICENSE)

## Acknoledgements ##
This project uses the [x11-rs](https://github.com/Daggerbot/x11-rs) bindings. Thanks to [Daggerbot](https://github.com/Daggerbot) !
