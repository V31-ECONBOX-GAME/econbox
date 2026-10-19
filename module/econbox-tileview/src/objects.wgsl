struct Wind {
    clock: vec4<f32>,
    leaves: vec4<f32>,
    celestial: vec4<f32>,
    tint: vec4<f32>,
};

struct FragmentInput {
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> wind: Wind;
@group(2) @binding(1) var atlas: texture_2d<f32>;
@group(2) @binding(2) var atlas_sampler: sampler;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let w = in.color.z;
    let h = in.color.w;
    let big_w = w + 2.0;
    let big_h = h + 10.0;
    var u = in.uv.x / big_w;
    var v = in.uv.y / big_h;

    let packed = in.tangent.y;
    let mode = packed - 4.0 * floor(packed / 4.0);
    let frames = floor(packed / 4.0);

    if mode > 0.5 {
        var params = wind.leaves;
        if mode > 1.5 {
            params = wind.celestial;
        }
        let t = wind.clock.x * params.w + in.tangent.x;
        let dx = u + params.x * params.y * sin(t + v);
        let dy = v + params.x * params.z * cos(t + dx);
        u = u + v * (dx - u);
        v = v + v * (dy - v);
    }

    let tx = floor(u * big_w) - 1.0;
    let ty = floor(v * big_h) - 1.0;
    if tx < 0.0 || ty < 0.0 || tx >= w || ty >= h {
        discard;
    }

    var frame = 0.0;
    if frames > 1.5 {
        let time = abs(wind.clock.y * in.tangent.z + in.tangent.w);
        frame = floor(time - frames * floor(time / frames));
    }

    let at = vec2<i32>(
        i32(in.color.x + frame * w + tx),
        i32(in.color.y + (h - 1.0 - ty)),
    );
    let pixel = textureLoad(atlas, at, 0);
    if pixel.a < 0.01 {
        discard;
    }
    return vec4(pixel.rgb * wind.tint.rgb, pixel.a);
}
