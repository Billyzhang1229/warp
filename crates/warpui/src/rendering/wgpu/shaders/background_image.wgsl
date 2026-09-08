fn adjust_background_color(sampled: vec4f, adjustments: vec4f) -> vec4f {
    let bright = sampled.rgb * adjustments.x;
    let color = clamp((bright - 0.5) * adjustments.y + 0.5, vec3f(0.0), vec3f(1.0));
    return vec4f(color, sampled.a);
}

fn shade_background_edges(sampled: vec4f, position: vec2f, viewport: vec2f, adjustments: vec4f, vignette: f32) -> vec4f {
    let uv = clamp(position / max(viewport, vec2f(1.0)), vec2f(0.0), vec2f(1.0));
    var shade = 1.0;
    if adjustments.z > 0.0 && adjustments.w < 1.0 {
        shade *= 1.0 - adjustments.z * smoothstep(adjustments.w, 1.0, uv.y);
    }
    if vignette > 0.0 {
        let edge = smoothstep(0.35, 1.41421356, length(uv * 2.0 - 1.0));
        shade *= 1.0 - vignette * edge;
    }
    return vec4f(sampled.rgb * shade, sampled.a);
}
