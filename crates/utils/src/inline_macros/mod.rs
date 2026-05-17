pub mod template_macros;

#[macro_export]
macro_rules! impl_deref {
    ($target:ident for $struct:ident) => {
        impl Deref for $struct {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.$target
            }
        }
    };
}

#[macro_export]
macro_rules! time_exec {
    ($block:expr) => {{
        let now = std::time::Instant::now();
        let result = $block;
        let elapsed = now.elapsed();
        println!("Timed at {:.2?}", elapsed);
        result
    }};
    ($name:literal, $block:expr) => {{
        let now = std::time::Instant::now();
        let result = $block;
        let elapsed = now.elapsed();
        println!("[{}]: Timed at {:.2?}", $name, elapsed);
        result
    }};
}
