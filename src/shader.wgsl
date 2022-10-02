
@group(0) @binding(0)
var<uniform> modelview: mat4x4<f32>;

struct VertOut {
    @builtin(position) position: vec4<f32>,
    @location(0) obj_color: vec3<f32>,
};

@vertex
fn vertex(
    @location(0) vert: vec3<f32>,
    @location(1) obj_color: vec3<f32>,
    @location(2) obj_pos: vec3<f32>,
    @location(3) obj_x: vec3<f32>,
    @location(4) obj_y: vec3<f32>,
    @location(5) obj_z: vec3<f32>,
) -> VertOut {

    // transform
    let world_vert = vec3<f32>(dot(vert, obj_x), dot(vert, obj_y), dot(vert, obj_z)) + obj_pos;
    let view_vert = vec4<f32>(world_vert, 1.0) * modelview;

    // emit vertex
    var vert: VertOut;
    vert.position = vec4<f32>(view_vert.xyz, 1.0);
    vert.obj_color = obj_color;
    return vert;
}

@fragment
fn fragment(vertex: VertOut) -> @location(0) vec4<f32> {
    return vec4<f32>(vertex.obj_color, 1.0);
}


