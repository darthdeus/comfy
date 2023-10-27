in vec2 TexCoords;
layout(location = 0) out vec4 color;

uniform vec3 iResolution;
uniform vec4 iMouse;
uniform float iTime;
uniform float iTimeDelta;
uniform int iFrame;
uniform float iFrameRate;

uniform sampler2D iChannel0;
uniform sampler2D iChannel1;
uniform sampler2D iChannel2;
uniform sampler2D iChannel3;

uniform vec2 offsets[9];
uniform int edge_kernel[9];
uniform float blur_kernel[9];

uniform bool chaos;
uniform bool confuse;
uniform bool shake;
uniform bool skip_pp;

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
///////////////////////////////////////////////
///////////////////////////////////////////////
///////////////////////////////////////////////


// Inspired by https://www.youtube.com/watch?v=8wOUe32Pt-E

// What power of 2 the pixel cell sizes are increased to
const int pixel_scale = 1;

// https://lospec.com/palette-list/oil-6
// Should be sorted in increasing order by perceived luminance for best

// Can work with up to 256 distinct colors
// const vec4[] palette = vec4[] (
// vec4(39./255., 39./255., 68./255., 1.),
// vec4(73./255., 77./255., 126./255., 1.),
// vec4(139./255., 109./255., 156./255.,1.),
// vec4(198./255., 159./255., 165./255., 1.),
// vec4(242./255., 211./255., 171./255., 1.),
// vec4(251./255., 245./255., 239./255., 1.));

// const vec4[] palette = vec4[] (
// vec4(246./255., 205./255., 38./255., 1.),
// vec4(172./255., 107./255., 38./255., 1.),
// vec4(86./255., 50./255., 38./255.,1.),
// vec4(51./255., 28./255., 23./255., 1.),
// vec4(187./255., 127./255., 87./255., 1.),
// vec4(114./255., 89./255., 86./255., 1.),
// vec4(57./255., 57./255., 57./255., 1.),
// vec4(32./255., 32./255., 32./255., 1.));

const vec4[] palette = vec4[] (
vec4(3./255., 2./255., 7./255., 1.),
vec4(10./255., 16./255., 25./255., 1.),
vec4(97./255., 25./255., 55./255.,1.),
vec4(187./255., 45./255., 85./255.,1.),
vec4(235./255., 119./255., 91./255., 1.),
vec4(248./255., 176./255., 101./255., 1.));



// Amount of colors in the palette
// Changing this is not recommended
const int colors = int(palette.length());

// How much the dither effect spreads. By default it is set to decrease as
// the amount of colors increases.
// Set to 0 to disable the dithering effect for flat color areas.
const float dither_spread = 1./float(colors);

// Precomputed threshold map for dithering
const mat4x4 threshold = mat4x4(0., 8., 2., 10.,
                                12., 4., 14., 6.,
                                3.,11.,1.,9.,
                                15.,7.,13., 5.);

// Chooses a color from the palette based on the current luminance
vec4 applyPalette(float lum)
{
    lum = floor(lum * float(colors));
    return palette[int(lum)];
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // https://luka712.github.io/2018/07/01/Pixelate-it-Shadertoy-Unity/
    float pixelSizeX = 1.0 / iResolution.x;
    float pixelSizeY = 1.0 / iResolution.y;
    float cellSizeX = pow(2., float(pixel_scale)) * pixelSizeX;
    float cellSizeY = pow(2., float(pixel_scale)) * pixelSizeY;

    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord/iResolution.xy;

    // Convert pixel coordinates to cell coordinates
    float u = cellSizeX * floor(uv.x / cellSizeX);
    float v = cellSizeY * floor(uv.y / cellSizeY);

    // get pixel information from the cell coordinates
    vec4 col = texture(iChannel0, vec2(u,v));

    // https://en.wikipedia.org/wiki/Ordered_dithering
    int x = int(u / cellSizeX) % 4;
    int y = int(v / cellSizeY) % 4;
    col.r = col.r + (dither_spread * ((threshold[x][y]/16.) -.5));
    col.g = col.g + (dither_spread * ((threshold[x][y]/16.) -.5));
    col.b = col.b + (dither_spread * ((threshold[x][y]/16.) -.5));
    col.r = floor(col.r * float(colors-1) + .5)/float(colors-1);
    col.g = floor(col.g * float(colors-1) + .5)/float(colors-1);
    col.b = floor(col.b * float(colors-1) + .5)/float(colors-1);

    // Calculate luminance
    float lum = (0.299*col.r + 0.587*col.g + 0.114*col.b);

    // Apply the new color pallet, if applicable
    col = applyPalette(lum);

    float gamma = 0.8;

    col.x = pow(col.x, gamma);
    col.y = pow(col.y, gamma);
    col.z = pow(col.z, gamma);

    // Output to screen
    fragColor = vec4(col);
}


// void mainImage(out vec4 fragColor, in vec2 fragCoord) {
//   fragColor = texture(iChannel0, TexCoords);
// }

void main() {
  vec4 a = texture(iChannel0, TexCoords);

  // bool v = false;
  //
  // if (v) {
  //   color = vec4(input_color);
  // } else {
  //   color = vec4(vec3(1.0 - input_color), 1.0);
  // }
  // color = input_color;

  vec4 b;
  mainImage(b, gl_FragCoord.xy);

  vec3 ah = rgb2hsv(a.rgb);
  vec3 bh = rgb2hsv(b.rgb);

  if (skip_pp) {
    color = a;
  } else {
    // vec3 hcol;
    //
    // if (ah.y > bh.y) {
    //   hcol = vec3(ah.x, bh.yz);
    // } else {
    //   hcol = vec3(bh.x, ah.yz);
    // }

    // color = vec4(hsv2rgb(hcol), 1.0);
    // color = vec4(input_color.xz, gen_color.y, 1.0);

    color = a;
  }
}
