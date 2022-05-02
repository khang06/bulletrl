use backend::{Backend, MinifbBackend, TcpBackend};

mod backend;
mod game;
mod util;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args: Vec<String> = std::env::args().collect();
    let mut backend: Box<dyn Backend> = if args.len() > 1 {
        Box::new(TcpBackend::new(
            args[1].parse().expect("failed to parse port"),
        ))
    } else {
        Box::new(MinifbBackend::default())
    };
    backend.main_loop();
}
