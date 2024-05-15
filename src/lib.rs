use rhai::plugin::*;

#[export_module]
mod rhai_http {
    // Build your module here.
}

/// Export the rhai_http module.
#[no_mangle]
pub extern "C" fn module_entrypoint() -> rhai::Shared<rhai::Module> {

    // The seed must be the same as the one used in the program that will
    // load this module.
    rhai::config::hashing::set_ahash_seed(Some([1, 2, 3, 4])).unwrap();

    rhai::exported_module!(rhai_http).into()
}