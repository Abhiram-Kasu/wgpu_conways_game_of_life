@group(0) @binding(0) var<uniform> dimensions: vec2<u32>;

@group(0) @binding(1) var src: texture_storage_2d<r32uint, read>;

@group(0) @binding(2) var dst: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= dimensions.x || id.y >= dimensions.y {
        return;
    }

    let selfCell = textureLoad(src, vec2<u32>(id.xy)).r;
    let neighbors = countNeighbors(vec2<u32>(id.xy));

    var newState: u32 = 0u;
    if selfCell == 1u {
        if neighbors == 2u || neighbors == 3u {
            newState = 1u;
        }
    } else {
        if neighbors == 3u {
            newState = 1u;
        }
    }

    textureStore(dst, vec2<i32>(id.xy), vec4<u32>(newState, 0u, 0u, 0u));
}

fn countNeighbors(coord: vec2<u32>) -> u32 {
    var count: u32 = 0u;
    let width = dimensions.x;
    let height = dimensions.y;

    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            if x == 0 && y == 0 {
                continue;
            }

            let nx = i32(coord.x) + x;
            let ny = i32(coord.y) + y;
            let wrappedX = (nx + i32(width)) % i32(width);
            let wrappedY = (ny + i32(height)) % i32(height);

            let neighborCoord = vec2<u32>(u32(wrappedX), u32(wrappedY));
            let neighborCell = textureLoad(src, neighborCoord).r;
            count = count + neighborCell;
        }
    }
    return count;
}
