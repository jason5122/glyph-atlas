#define int_t int
#define float_t float
#define vec3_t vec3

in vec2 TexCoords;
flat in vec4 fg;
flat in vec4 bg;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

#define FRAG_COLOR color
#define ALPHA_MASK alphaMask

#define COLORED 1

uniform int_t renderingPass;
uniform sampler2D mask;

void main() {
    if (renderingPass == 0) {
        if (bg.a == 0.0) {
            discard;
        }

        ALPHA_MASK = vec4(1.0);
        // Premultiply background color by alpha.
        FRAG_COLOR = vec4(bg.rgb * bg.a, bg.a);
        return;
    }

    float_t colored = fg.a;

    // The wide char information is already stripped, so it's safe to check for equality here.
    if (int(colored) == COLORED) {
        // Color glyphs, like emojis.
        FRAG_COLOR = texture(mask, TexCoords);
        ALPHA_MASK = vec4(FRAG_COLOR.a);

        // Revert alpha premultiplication.
        if (FRAG_COLOR.a != 0.0) {
            FRAG_COLOR.rgb = vec3(FRAG_COLOR.rgb / FRAG_COLOR.a);
        }

        FRAG_COLOR = vec4(FRAG_COLOR.rgb, 1.0);
    } else {
        // Regular text glyphs.
        vec3_t textColor = texture(mask, TexCoords).rgb;
        ALPHA_MASK = vec4(textColor, textColor.r);
        FRAG_COLOR = vec4(fg.rgb, 1.0);
    }
}