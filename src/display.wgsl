struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32>;
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }

    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = (pos + 1.0) * 0.5;

    return out;
}

@group(0) @binding(0) var game_texture: texture_2d<u32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = textureDimensions(game_texture);
    let uv = in.tex_coords;
    let pixel_coords = vec2<i32>(uv * vec2<f32>(dims));

    var sum: f32 = 0.0;
    var count: f32 = 0.0;

    let blur_amm = 0;
    // Average 3x3 neighborhood
    for (var y: i32 = -blur_amm; y <= blur_amm; y = y + 1) {
        for (var x: i32 = -blur_amm; x <= blur_amm; x = x + 1) {
            let sample_coords = clamp(pixel_coords + vec2<i32>(x, y), vec2<i32>(0, 0), vec2<i32>(dims) - vec2<i32>(1, 1));
            let sample_val = f32(textureLoad(game_texture, sample_coords, 0).r);
            sum = sum + sample_val;
            count = count + 1.0;
        }
    }

    let avg = sum / count;
    let avgM = mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), vec4<f32>(.71, 0.58, 0.06, 1.0), avg);



    return avgM;
}
