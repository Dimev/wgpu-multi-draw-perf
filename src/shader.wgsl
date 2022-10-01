
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
    @location(2) obj_transform: mat4x4<f32>,
) -> VertexOut {
    var vert: VertexOut;
    vert.position = vert;
    vert.obj_color = obj_color;
    return vert;
}

@framgent
fn fragment(vertex: VertOut) -> @location(0) vec4<f32> {
    return vec4<f32>(vertex.obj_color, 1.0);
}


