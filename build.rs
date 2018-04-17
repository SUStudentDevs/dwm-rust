fn main() {
    println!("cargo:rustc-flags=-L /usr/X11/lib -l X11 -l stdc++ -l Xft");
}
