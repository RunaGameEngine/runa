struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>, // fragment shader ждёт
};

@vertex
fn vs_main(@location(0) a_position: vec2<f32>,
           @location(1) a_tex_coords: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(a_position, 0.0, 1.0);
    out.tex_coords = a_tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 1.0); // тестовый красный спрайт
}
