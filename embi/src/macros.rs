#[macro_export]
macro_rules! define_main {
    ($run:ident) => {
        fn main() {
            #[cfg(feature = "color-backtrace")]
            color_backtrace::install();

            #[cfg(not(target_arch = "wasm32"))]
            {
                pollster::block_on($run());
            }

            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local($run());
            }
        }
    }
}
