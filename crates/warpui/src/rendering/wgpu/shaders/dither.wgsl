// Adapted from capy-landing/lib/dither/portrait.wgsl.
@group(2) @binding(0) var blueNoise: texture_2d<f32>;

fn dither_background(uv: vec2f, position: vec2f, resolution: vec2f, effect: vec4f) -> vec4f {
  // Cover crop, biased toward the face on tall displays.
    let screenAspect = resolution.x / resolution.y;
    let imageAspect = vec2f(textureDimensions(imageTexture)).x / vec2f(textureDimensions(imageTexture)).y;
    let scale = vec2f(min(screenAspect / imageAspect, 1.0), min(imageAspect / screenAspect, 1.0));
    let anchor = vec2f(select(0.5, 0.69, screenAspect < 1.0), 0.42);
    let breath = sin(effect.x * 0.22) * 0.004 * effect.w;
    var sampleUV = (uv - 0.5) * scale * (0.975 + breath) + anchor * (1.0 - scale) + scale * 0.5;
    sampleUV += vec2f(sin(effect.x * 0.12) * 0.003, cos(effect.x * 0.16) * 0.002) * effect.w;
    let sampled = textureSampleLevel(imageTexture, imageSampler, sampleUV, 0.0);
    let source = sampled.rgb;
  // Pixel-like dots and plus signs, rather than circular halftone cells.
  // A travelling wave moves the threshold and bends the screen slightly;
  // the portrait itself remains legible and does not ripple like liquid.
  // Domain-warped, interfering contours: no single repeating diagonal band.
    let space = uv * vec2f(screenAspect, 1.0);
    let warp = space + vec2f(
        sin(space.y * 6.3 + effect.x * 0.33) + cos(space.x * 4.1 - space.y * 3.0 - effect.x * 0.27),
        cos(space.x * 5.2 - effect.x * 0.29) + sin(space.y * 4.4 + space.x * 2.5 + effect.x * 0.21)
    ) * 0.16;
    let organic = sin(warp.x * 8.0 + warp.y * 5.0 - effect.x * 0.85) + sin(warp.y * 10.5 - warp.x * 4.0 + effect.x * 0.63) * 0.48 + cos(length(warp - vec2f(0.35, 0.4)) * 12.0 - effect.x * 0.74) * 0.30;
    let wave = clamp(organic * 0.67, -1.3, 1.3) * effect.w;
    let drift = vec2f(sin(warp.y * 7.0 - effect.x * 0.6), cos(warp.x * 8.0 + effect.x * 0.5)) * 0.18 * effect.w;
    let grid = position / effect.z + drift;
    let cell = vec2i(floor(grid));
    let noiseSize = vec2i(textureDimensions(blueNoise));
    let noisePixel = ((cell % noiseSize) + noiseSize) % noiseSize;
    let noise = textureLoad(blueNoise, noisePixel, 0).r;
    let lit = source * (0.98 + wave * 0.12);
    let luminance = dot(lit, vec3f(0.2126, 0.7152, 0.0722));
    let tone = clamp(luminance * 1.8, 0.0, 1.0);
  // Adjacent cells alternate their transition thresholds: a wave visibly
  // transforms sparse dots into crosses and back, without a 4x4 block boundary.
    let parity = f32((cell.x + cell.y) & 1);
    let threshold = mix(0.23, 0.68, parity) + (noise - 0.5) * 0.14;
    let crossAmount = smoothstep(threshold - 0.16, threshold + 0.16, tone + wave * 0.32);
    let arm = mix(0.105, 0.37, crossAmount);
    let thickness = 0.09 + tone * 0.025;
    let p = abs(fract(grid) - 0.5);
    let shapeDistance = min(max(p.x - arm, p.y - thickness), max(p.x - thickness, p.y - arm));
    let aa = 0.45 / effect.z;
    let mark = (1.0 - smoothstep(-aa, aa, shapeDistance)) * smoothstep(0.004, 0.025, luminance);
    let ink = clamp(lit * 1.35 + vec3f(0.17, 0.13, 0.23), vec3f(0.0), vec3f(1.0));
    let screened = lit * 0.40 + ink * mark;
    let color = mix(lit, screened, effect.y);
    let vignette = 1.0 - smoothstep(0.32, 0.88, distance(uv, vec2f(0.5, 0.38))) * 0.55;
    let fade = 1.0 - smoothstep(0.46, 1.0, uv.y) * 0.94;
    return vec4f(color * vignette * fade * 0.86, sampled.a);
}
