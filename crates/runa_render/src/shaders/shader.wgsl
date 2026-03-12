struct VertexInput {
    @location(0) position: vec3<f32>,   // локальная позиция вершины квада (-0.5..0.5)
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InstanceData {
    @location(2) position: vec3<f32>,   // мировая позиция спрайта
    @location(3) rotation: f32,         // поворот вокруг оси Z (радианы)
    @location(4) scale: vec3<f32>,      // масштаб по осям X, Y, Z
    @location(5) uv_offset: vec2<f32>,
    @location(6) uv_size: vec2<f32>,
    @location(7) flip: u32,
    @location(8) _pad: f32,             // паддинг до 32 байт
};

struct Globals {
    view_proj: mat4x4<f32>,
    aspect: f32,
    _padding: vec3<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_sampler: sampler;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceData,
) -> VertexOutput {
    var out: VertexOutput;

    // === 1. Применяем 2D-поворот вокруг оси Z ===
    let cos_a = cos(instance.rotation);
    let sin_a = sin(instance.rotation);

    let rotated_x = vertex.position.x * cos_a - vertex.position.y * sin_a;
    let rotated_y = vertex.position.x * sin_a + vertex.position.y * cos_a;

    // === 2. Применяем масштаб ===
    let scaled_x = rotated_x * instance.scale.x;
    let scaled_y = rotated_y * instance.scale.y;
    let scaled_z = vertex.position.z * instance.scale.z; // обычно 0.0 для спрайтов

    // === 3. Добавляем мировую позицию инстанса ===
    let world_x = scaled_x + instance.position.x;
    let world_y = scaled_y + instance.position.y;
    let world_z = scaled_z + instance.position.z;

    // === 4. Коррекция аспекта для пиксель-перфект рендеринга ===
    let corrected_x = world_x * globals.aspect;

    // === 5. Преобразуем в клип-пространство ===
    out.clip_position = globals.view_proj * vec4<f32>(corrected_x, world_y, world_z, 1.0);

    // Базовые UV от вершины (0..1)
    var uv = vertex.tex_coords;

    // Флип
    if (instance.flip & 1u) != 0u { uv.x = 1.0 - uv.x; }
    if (instance.flip & 2u) != 0u { uv.y = 1.0 - uv.y; }

    // ✅ КЛЮЧЕВАЯ СТРОКА: применяем UV-rect
    let final_uv = instance.uv_offset + uv * instance.uv_size;

    out.tex_coords = final_uv;  // ← Передаём в фрагментный шейдер

    // ... остальная логика позиции ...
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_sampler, in.tex_coords);
}
