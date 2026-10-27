fn main() {
    let artifacts = xmlsec1_src::Build::new().build();
    artifacts.print_cargo_metadata();
}
