use ParticleLife3D::run;


const WIDTH: i32 = 600;
const HEIGHT: i32 = 600;



fn main() {
    // env_logger::init();
    pollster::block_on(run(WIDTH, HEIGHT));
}