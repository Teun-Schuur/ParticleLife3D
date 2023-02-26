use ParticleLife3D::run;


const WIDTH: i32 = 1700;
const HEIGHT: i32 = 1000;



fn main() {
    // env_logger::init();
    pollster::block_on(run(WIDTH, HEIGHT));
}