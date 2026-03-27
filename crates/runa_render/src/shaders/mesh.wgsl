// ===== UNIFORMS =====
struct MeshUniforms {
    view_proj: mat4x4<f32>,  // 64 байта
    view: mat4x4<f32>,       // 64 байта
    color: vec4<f32>,        // 16 байт - цвет меша
    _padding: array<vec4<f32>, 7>, // 112 байт для выравнивания до 256
};
@group(0) @binding(0) var<uniform> globals: MeshUniforms;

// ===== TEXTURES (optional - for textured meshes) =====
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_sampler: sampler;

// ===== VERTEX SHADER =====
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // Мировая позиция вершины
    let world_pos = in.position;

    // Мировая нормаль
    let world_normal = normalize(in.normal);

    // Финальная позиция в клип-пространстве
    var out: VertexOutput;
    out.position = globals.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = in.uv;
    out.normal = world_normal;
    out.world_pos = world_pos;

    return out;
}

// ===== ФРАГМЕНТНЫЙ ШЕЙДЕР =====
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Простое диффузное освещение (направленный свет)
    let light_dir = normalize(vec3<f32>(1.0, -1.0, 1.0));
    let normal = normalize(in.normal);
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.8 + 0.2; // ambient + diffuse

    // Используем цвет из uniform
    let base_color = globals.color;

    // Применяем освещение
    let final_color = base_color * diffuse;

    return final_color;
}
