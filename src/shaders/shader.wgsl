
// Vertex shader
const SIZE: f32 = 1.1403000; // in nm

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Particle {
    @location(3) pos: vec2<f32>,
    @location(6) color: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(4) color: vec3<f32>,
};


@vertex
fn vs_main(model: VertexInput, particle: Particle) -> VertexOutput {
    var out: VertexOutput;
    out.color = particle.color;
    var model_pos = vec3<f32>(particle.pos, 0.0);
    var model_matrix = mat4x4<f32>(
        vec4<f32>(SIZE, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, SIZE, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, SIZE, 0.0),
        vec4<f32>(model_pos, 1.0),
    );

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
