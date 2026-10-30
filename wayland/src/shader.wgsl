struct Uniforms {
    view_size: vec2<f32>,
    scale: f32,
    time: f32,
    column_width: f32,
    column_count: i32,
    pile_top: f32,
    occluder_count: i32,
    origin: vec2<f32>,
    pile_bottom: f32,
    height_offset: i32,
    light: vec4<f32>,
    shadow_tint: vec4<f32>,
    occluders: array<vec4<f32>, 16>,
    motion_blur: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> heights: array<f32>;

struct SpriteIn {
    @location(0) position: vec2<f32>,
    @location(1) velocity: vec2<f32>,
    @location(2) size: f32,
    @location(3) alpha: f32,
    @location(4) soft: f32,
    @location(5) tint_amount: f32,
    @location(6) tint_color: vec3<f32>,
    @location(7) lump: f32,
}

struct SpriteOut {
    @builtin(position) position: vec4<f32>,
    @location(0) coord: vec2<f32>,
    @location(1) alpha: f32,
    @location(2) soft: f32,
    @location(3) tint: vec4<f32>,
    @location(4) streak: vec2<f32>,
    @location(5) radius: f32,
    @location(6) lump: f32,
    @location(7) spin: f32,
}

@vertex
fn sprite_vertex(
    @builtin(vertex_index) vid: u32,
    sprite: SpriteIn,
) -> SpriteOut {
    let pixels = sprite.size * uniforms.scale;
    let mb = uniforms.motion_blur.x;
    let is_cloud = sprite.soft > 0.8;
    let travel = sprite.velocity * (0.018 * mb);
    // Cloud motes are diffuse billowing gas puffs: they must not streak like sharp rain/ice needles
    let smear = select(min(length(travel), sprite.size * select(2.2, 1.2, sprite.lump > 0.0) * mb), 0.0, is_cloud);
    var streak = vec2<f32>(0.0, 0.0);
    if (mb > 0.01 && smear > 0.25) {
        streak = normalize(travel) * smear * uniforms.scale;
    }
    // Generous quad reach for soft Gaussian-like cloud motes so alpha decays to 0.0 well inside the quad
    let reach = select(select(1.05, 1.35, sprite.lump > 0.0), 1.45, is_cloud);
    let extent = max(pixels * reach + length(streak), 2.0);
    let corner = vec2<f32>(f32(vid & 1u), f32(vid >> 1u));
    
    var out: SpriteOut;
    let center = sprite.position / uniforms.view_size * 2.0 - vec2<f32>(1.0, 1.0);
    let offset = (corner - vec2<f32>(0.5, 0.5)) * extent * 2.0 / (uniforms.view_size * uniforms.scale);
    out.position = vec4<f32>(center + offset, 0.0, 1.0);
    out.coord = vec2<f32>(corner.x, 1.0 - corner.y);
    out.streak = streak / extent;
    out.radius = pixels / extent * 0.5;
    out.alpha = sprite.alpha * min(pixels / 2.0, 1.0) * pixels / max(pixels + length(streak) * 0.6, 0.001);
    out.soft = sprite.soft;
    out.tint = vec4<f32>(sprite.tint_color, sprite.tint_amount);
    out.lump = sprite.lump;
    out.spin = select(0.0, uniforms.time * (sprite.lump - 0.5) * 9.0 + sprite.lump * 40.0, sprite.lump > 0.0);
    return out;
}

@fragment
fn sprite_fragment(in: SpriteOut) -> @location(0) vec4<f32> {
    let p = vec2<f32>(in.coord.x - 0.5, 0.5 - in.coord.y);
    let half_ = in.streak * 0.5;
    let span = dot(half_, half_);
    var t = 0.0;
    if (span > 0.0) {
        t = clamp(dot(p, half_) / span, -1.0, 1.0);
    }
    let d = p - half_ * t;
    var r = length(d) / max(in.radius, 0.001);
    var shading = 0.0;
    if (in.lump > 0.0) {
        let angle = atan2(d.y, d.x) + in.spin;
        r /= 1.0 + 0.14 * sin(3.0 * angle) + 0.08 * sin(5.0 * angle + in.lump * 21.0) + 0.05 * sin(2.0 * angle + in.lump * 9.0);
        shading = clamp(dot(d / max(in.radius, 0.001), vec2<f32>(0.45, -0.6)), 0.0, 1.0) * 0.4;
    }
    var head = 1.0;
    if (span > 0.0) {
        head = 0.15 + 0.85 * pow(t * 0.5 + 0.5, 1.5);
    }
    let a = (1.0 - smoothstep(mix(0.6, 0.0, in.soft), 1.0, r)) * in.alpha * head;
    if (a < 0.002) {
        discard;
    }
    let core = vec3<f32>(0.965, 0.975, 0.995);
    let rim = vec3<f32>(0.62, 0.67, 0.77);
    var color = mix(core, rim, clamp(smoothstep(0.35, 1.0, r) * 0.65 + shading, 0.0, 1.0));
    color = mix(color, in.tint.rgb, in.tint.a) * uniforms.light.rgb;
    return vec4<f32>(color * a, a);
}

struct PileOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
}

@vertex
fn pile_vertex(@builtin(vertex_index) vid: u32) -> PileOut {
    let corner = vec2<f32>(f32(vid & 1u), f32(vid >> 1u));
    let p = vec2<f32>(corner.x * f32(uniforms.column_count) * uniforms.column_width, mix(uniforms.pile_bottom, uniforms.pile_top, corner.y));
    var out: PileOut;
    let world = p + uniforms.origin;
    let ndc = world / uniforms.view_size * 2.0 - vec2<f32>(1.0, 1.0);
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.local = p;
    return out;
}

fn height_at(start: i32, x: f32) -> f32 {
    let f = x / uniforms.column_width - 0.5;
    let base = floor(f);
    let i = clamp(i32(base), 0, uniforms.column_count - 1);
    let j = min(i + 1, uniforms.column_count - 1);
    return mix(heights[start + i], heights[start + j], clamp(f - base, 0.0, 1.0));
}

fn hash2(p_in: vec2<f32>) -> f32 {
    var p = fract(p_in * vec2<f32>(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

@fragment
fn pile_fragment(in: PileOut) -> @location(0) vec4<f32> {
    let world = in.local + uniforms.origin;
    for (var k = 0; k < uniforms.occluder_count; k = k + 1) {
        let o = uniforms.occluders[k];
        if (world.x >= o.x && world.x < o.z && world.y >= o.y && world.y < o.w) {
            discard;
        }
    }

    let x = in.local.x;
    let level = height_at(uniforms.height_offset, x);
    let bed = height_at(uniforms.height_offset + uniforms.column_count, x);
    let depth = level - in.local.y;
    let coverage = clamp(depth * uniforms.scale + 0.5, 0.0, 1.0)
        * clamp((in.local.y - bed) * uniforms.scale + 0.5, 0.0, 1.0)
        * clamp((level - bed) / 0.75, 0.0, 1.0);
    if (coverage <= 0.0) {
        discard;
    }

    let slope = (height_at(uniforms.height_offset, x + 2.0) - height_at(uniforms.height_offset, x - 2.0)) * 0.25;
    let lit = dot(normalize(vec2<f32>(-slope, 1.0)), normalize(vec2<f32>(-0.5, 1.0)));

    let shadow = mix(vec3<f32>(0.72, 0.78, 0.88), uniforms.shadow_tint.rgb, uniforms.shadow_tint.a);
    let surface = mix(vec3<f32>(0.925, 0.94, 0.965), min(uniforms.shadow_tint.rgb * 1.18, vec3<f32>(1.0, 1.0, 1.0)), uniforms.shadow_tint.a * 0.2);
    let shade = (1.0 - exp(-depth / 30.0)) * 0.32 + (1.0 - lit) * 0.6 * exp(-depth / 5.0);
    var color = mix(surface, shadow, clamp(shade, 0.0, 1.0));
    color = mix(color, vec3<f32>(0.975, 0.982, 0.995), exp(-depth * 0.9) * 0.6);
    color += (hash2(floor(in.position.xy)) - 0.5) * 0.02;

    let seed = hash2(floor(in.position.xy * 0.5) + 31.7);
    if (seed > 0.992) {
        let twinkle = pow(clamp(sin(uniforms.time * (0.6 + seed * 1.8) + seed * 90.0), 0.0, 1.0), 28.0);
        color = mix(color, vec3<f32>(1.0, 1.0, 1.0), twinkle * exp(-depth / 7.0));
    }

    color *= uniforms.light.rgb;
    return vec4<f32>(color * coverage, coverage);
}
