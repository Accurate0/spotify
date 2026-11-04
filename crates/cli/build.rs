fn main() {
    println!("cargo:rerun-if-changed=../../schema.graphql");
    cynic_codegen::register_schema("spotify")
        .from_sdl_file("../../schema.graphql")
        .expect("schema.graphql is readable")
        .as_default()
        .expect("schema registers");
}
