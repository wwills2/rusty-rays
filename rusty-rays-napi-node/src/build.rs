use napi_build;

fn main() {
    // Generates proper metadata so Node can load the .node binary
    napi_build::setup();
}
