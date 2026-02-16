// ===== UNIFORMS =====
struct MeshUniforms {
    view_proj: mat4x4<f32>,  // 64 байта
    view: mat4x4<f32>,       // 64 байта
    // Используем массив из 8 vec4 (каждый 16 байт) вместо 32 float
    _padding: array<vec4<f32>, 8>, // 128 байт (8 × 16)
};
@group(0) @binding(0) var<uniform> globals: MeshUniforms;

// ===== ТЕКСТУРЫ =====
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_sampler: sampler;

// ===== ВЕРШИННЫЙ ШЕЙДЕР =====
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    // Инстансинг
    @location(3) instance_row0: vec4<f32>,
    @location(4) instance_row1: vec4<f32>,
    @location(5) instance_row2: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // Собираем матрицу инстанса из строк
    var instance_matrix: mat4x4<f32>;
    instance_matrix[0] = vec4<f32>(in.instance_row0.xyz, 0.0);
    instance_matrix[1] = vec4<f32>(in.instance_row1.xyz, 0.0);
    instance_matrix[2] = vec4<f32>(in.instance_row2.xyz, 0.0);
    instance_matrix[3] = vec4<f32>(in.instance_row0.w, in.instance_row1.w, in.instance_row2.w, 1.0);

    // Мировая позиция вершины
    let world_pos = (instance_matrix * vec4<f32>(in.position, 1.0)).xyz;

    // Мировая нормаль (без трансляции)
    let world_normal = normalize((instance_matrix * vec4<f32>(in.normal, 0.0)).xyz);

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

    // Текстура
    let tex_color = textureSample(t_diffuse, s_sampler, in.uv);

    // Применяем освещение
    let final_color = tex_color * diffuse;

    return final_color;
}
